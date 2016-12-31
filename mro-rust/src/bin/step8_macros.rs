extern crate mal;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate env_logger;
use log::LogLevel::Trace;

use std::io::{self, Write};
use std::collections::HashMap;
use std::ops::Deref;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use std::env;
use std::path::Path;

use mal::reader;
use mal::printer;
use mal::env::{EnvType, Env, Symbol};

use mal::common::MalData;
use mal::common::MapKey;
use mal::common::NativeFunction;
use mal::common::{FnClosure, CallableFun, FunContext};
use mal::common::{mal_list_from_vec, mal_str_symbol, mal_symbol_name, mal_list_from_slice, is_mal_list};

use mal::core::init_ns_map;

#[derive(Debug, Clone)]
enum EvalError {
    General(String)
}

impl From<&'static str> for EvalError {
    fn from(err: &str) -> Self {
        EvalError::General(err.to_string())
    }
}

impl From<String> for EvalError {
    fn from(err: String) -> Self {
        EvalError::General(err)
    }
}

impl<'e> fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EvalError::General(ref err_msg) => {
                write!(f, "eval error: {}", err_msg)
            }
        }
    }
}

fn read<'a>(input: &'a str) -> Result<MalData, String> {
    reader::read_str(input)
}

fn call_function(env: EnvType, f: & NativeFunction, args: & [MalData]) -> Result<MalData, EvalError> {
    debug!("call_function, f: {:?}, args: {:?}", f, args);

    let callable = f.callable.clone();
    let ctx = &FunContext { eval: Some(make_eval_closure(env)), env: None };
    let result = callable(ctx, args);

    if log_enabled!(Trace) {
        for arg in args { trace!("arg: {:?}", arg) };
    }

    trace!("call_function, result: {:?}", result);
    result.map_err( | e | EvalError::from(e))
}


fn mal_closure(env: EnvType, binds: &Vec<Symbol>, body: &MalData) -> MalData {
    MalData::FnClosure(FnClosure::new(env, &binds, body))
}

fn eval_fn(env: EnvType, binds: & MalData, body: & MalData) -> Result<MalData, EvalError> {
    debug!("eval_fn, binds: {:?}, body: {:?}", binds, body);

    match binds {
        &MalData::Vector(ref b) | &MalData::List(ref b) => {
            let mut binds_vec = Vec::with_capacity(b.len());

            for bind in b.iter() {
                if let &MalData::Symbol(ref sym) = bind {
                    binds_vec.push(sym.clone());
                } else {
                    return Err(EvalError::General(format!("Expected symbol for bind, got: {:?}", bind)));
                }
            }

            Ok(mal_closure(env, &binds_vec, body))
        }

        _ =>
            Err(EvalError::General("expected vector for binds".to_string()))
    }
}

fn eval_def(env: EnvType, name: Option<&MalData>, value: Option<&MalData>) -> Result<MalData, EvalError> {
    match ( name, value ) {
        ( None, None ) | ( None, Some(_) ) | ( Some(_), None ) =>
            Err(EvalError::General("def! requires a name and a value".to_owned())),

        ( Some(& MalData::Symbol(ref key)), Some(value) ) => {
            eval(env.clone(), value).map( | evaluated | {
                env.borrow_mut().set(&key, &evaluated); 
                // debug!("def! {:?} -> {:?}/{:?}, env: {:?}", key, value, evaluated, env);
                evaluated
            })
        }

        ( key, value ) => {
            let err_msg = format!("unhandled in def!, key: {:?}, value: {:?}", key, value);
            Err(EvalError::General(err_msg.to_owned()))
        }
    }
}

fn eval_defmacro(env: EnvType, name: Option<&MalData>, value: Option<&MalData>) -> Result<MalData, EvalError> {
    match ( name, value ) {
        ( Some(& MalData::Symbol(ref name)), Some(ref fnc_form) ) => {
            let eval_res = eval(env.clone(), fnc_form);

            if let MalData::FnClosure(ref fnc) = eval_res? {
                let res = MalData::FnClosure(fnc.to_macro());

                env.borrow_mut().set(&name, &res); 
                Ok(res)
            } else {
                Err(EvalError::General("closure expected".to_owned()))
            }
        }

        ( None, None ) | ( None, Some(_) ) | ( Some(_), None ) =>
            Err(EvalError::General("defmacro! requires a name and a closure".to_owned())),

        ( key, value ) => {
            let err_msg = format!("unhandled in def!, key: {:?}, value: {:?}", key, value);
            Err(EvalError::General(err_msg.to_owned()))
        }
    }
}

fn is_pair(ast: &MalData) -> bool {
    match ast {
        &MalData::List(ref list) | &MalData::Vector(ref list) =>
            !list.is_empty(),

        _ =>
            false
    }
}

fn get_wrapped_list(ast: &MalData) -> Option<&Vec<MalData>> {
    match ast {
        &MalData::List(ref list) | &MalData::Vector(ref list) =>
            Some(list),

        _ =>
            None
    }
}

fn is_symbol_named(ast: &MalData, name: &str) -> bool {
    if let &MalData::Symbol(ref sym) = ast {
        sym == name
    } else {
        false
    }
}

fn quasiquote(ast: &MalData) -> Result<MalData, EvalError> {
    debug!("quasiquote, ast: {:?}", ast);

    if !is_pair(ast) {
        let res = mal_list_from_vec(vec![mal_str_symbol("quote"), ast.clone()]);
        debug!("quasiquote, !pair; res: {:?}", res);
        return Ok(res);
    } else if let Some(list) = get_wrapped_list(ast) {
        let first = list.first().unwrap();
        let first_as_list = if is_pair(first) { get_wrapped_list(first) } else { None };

        if mal_symbol_name(first).map_or(false, |sym| sym == "unquote") {
            return Ok(list.get(1).ok_or("expected form to unquote")?.clone())
        } else if is_pair(first) && first_as_list.map_or(false, |l| l.first().map_or(false, |f| is_symbol_named(f, "splice-unquote"))) {
            let to_unquote = first_as_list.map( |l| l.get(1) ).ok_or("expected form to unquote")?.unwrap();
            let quasiquote_rest = quasiquote(&mal_list_from_slice(&list[1..]))?;

            return Ok(mal_list_from_vec(vec![mal_str_symbol("concat"), to_unquote.clone(), quasiquote_rest]))
        } else {
            let quasiquote_first = quasiquote(&first)?;
            let quasiquote_rest = quasiquote(&mal_list_from_slice(&list[1..]))?;

            return Ok(mal_list_from_vec(vec![mal_str_symbol("cons"), quasiquote_first, quasiquote_rest]));
        }
    } else {
        panic!("quasiquote, ast: {:?}", ast);
    }
}

fn mal_list_head(ast: &MalData) -> Option<&MalData> {
    if let &MalData::List(ref list) = ast {
        list.first()
    } else {
        None
    }
}

fn is_macro_call(env: EnvType, ast: &MalData) -> bool {
    let list_head = mal_list_head(ast);

    let res = if let Some(& MalData::Symbol(ref sym)) = list_head {
        Env::get(&env, sym).map_or(false, |symval| {
            if let & MalData::FnClosure(ref fnc) = symval.as_ref() {
                fnc.is_macro()
            } else {
                false
            }
            
        })
    } else {
        false
    };

    debug!("is_macro_call, ast: {:?}, res: {:?}", ast, res);

    res
}

fn wrapped_env_type(env: Env) -> EnvType {
    Rc::from(RefCell::from(env))
}

fn mal_closure_apply(env: EnvType, closure: &FnClosure, args: &[MalData]) -> Result<MalData, EvalError> {
    debug!("mal_closure_apply, closure: {:?}, args: {:?}", closure, args);

    let fn_env = Env::new(Some(env.clone()), closure.binds.as_slice(), args)?;
    let res = eval(wrapped_env_type(fn_env), closure.body.as_ref());

    debug!("mal_closure_apply, res: {:?}", res);

    res
}

fn macroexpand(env: EnvType, ast: &MalData) -> Result<MalData, EvalError> {
    debug!("macroexpand, ast: {:?}", ast);

    let mut expand_ast = ast.clone();

    while is_macro_call(env.clone(), &expand_ast) {
        let list_head = mal_list_head(ast);

        debug!("macro call, list_head: {:?}", list_head);

        if let Some(& MalData::Symbol(ref sym)) = list_head {
            Env::get(&env, sym).map( |symval| {
                if let & MalData::FnClosure(ref fnc) = symval.as_ref() {
                    let empty_args = vec![];
                    let macro_args: &[MalData] = get_wrapped_list(ast).map_or(&empty_args, |l| &l[1..]);

                    debug!("macro call, fnc: {:?}, args: {:?}", fnc, macro_args);

                    expand_ast = mal_closure_apply(env.clone(), fnc, macro_args).unwrap();
                } else {
                    panic!("in macro call, not a closure: {:?}", symval);
                };
            });
        } else {
            panic!("in macro call, expected symbol, got: {:?}", list_head);
        };
    }

    Ok(expand_ast)
}

fn eval(mut env: EnvType, ast: & MalData) -> Result<MalData, EvalError> {
    let mut tco_ast: MalData = ast.clone();

    loop {
        let let_body: &MalData;

        debug!("eval, loop; tco_ast: {:?}", tco_ast);

        // macro expansion
        if is_mal_list(&tco_ast) {
            tco_ast = macroexpand(env.clone(), &tco_ast)?;

            if !is_mal_list(&tco_ast) {
                return eval_ast(env.clone(), &tco_ast);
            }
        }

        match tco_ast.clone() {
            MalData::List(ref list) if list.is_empty() =>
                return Ok(tco_ast.clone()),

            MalData::List(ref list) => {
                // sonderformen/special forms
                if let Some(& MalData::Symbol(ref sym)) = list.first() {
                    match sym.as_str() {
                        "def!" => {
                            debug!("eval, > def!");
                            let result = eval_def(env.clone(), list.get(1), list.get(2));
                            debug!("eval, < def!");
                            return result;
                        }

                        "defmacro!" => {
                            debug!("eval, > defmacro!");
                            let result = eval_defmacro(env.clone(), list.get(1), list.get(2));
                            debug!("eval, < defmacro!");
                            return result;
                        }

                        "let*" => {
                            debug!("eval, > let*");
                            let_body = list.get(2).unwrap();

                            let let_bindings = match list.get(1) {
                                Some(&MalData::List(ref bindings)) | Some(&MalData::Vector(ref bindings)) =>
                                    bindings,
                                _ =>
                                    return Err(EvalError::General("let* bindings".to_string())) // TODO
                            };

                            debug!("let body: {:?}", let_body);

                            let mut iter = let_bindings.iter();

                            {
                                // TODO binds, exprs
                                let let_env = wrapped_env_type(Env::new(Some(env.clone()), &[], &[])?);

                                loop {
                                    match ( iter.next(), iter.next() ) {
                                        ( Some(&MalData::Symbol(ref sym)), Some(ref def) ) => {
                                            let evaluated_def = eval(let_env.clone(), &def.clone())?;

                                            debug!("bind sym: {:?}, def: {:?} -eval-> {:?}", sym, def, evaluated_def);

                                            let_env.borrow_mut().set(&sym.clone(), &evaluated_def);
                                        }

                                        ( None, None ) => break,

                                        ( sym, def ) => {
                                            let err_msg = format!("error in let* binding; sym: {:?}, def: {:?}", sym, def);
                                            return Err(EvalError::General(err_msg))
                                        }
                                    }
                                }

                                // TCO: let-rumpf im folgenden schleifendurchgang evaluieren
                                env = let_env;
                                tco_ast = let_body.clone();
                                continue;
                            }

                            // debug!("eval, < let*, result: {:?}", result);
                            // return result;
                        }

                        "do" => {
                            trace!("eval, > do");
                            let mut result = Ok(MalData::Nil);
                            // liste aller mittels eval_ast zu evaluierender formen, letzte form wird hier im rahmen
                            // der TCO im folgenden schleifendurchgang evaluiert
                            let forms = &list[1..list.len() - 1];
                            trace!("eval_do, forms: {:?}", forms);

                            result = eval_ast(env.clone(), &MalData::List(Rc::from(forms.to_vec())));

                            tco_ast = list.last().unwrap().clone();  // TODO fehlerbehandlung
                            continue;
                        }

                        "if" => {
                            debug!("eval, > if");
                            let cond_form = &list[1];
                            let then_form = &list[2];

                            let cond = eval(env.clone(), cond_form);

                            match cond {
                                Ok(MalData::Nil)  | Ok(MalData::False)  => {
                                    if let Some(else_form) = list.get(3) {
                                        tco_ast = else_form.clone()
                                    } else {
                                        return Ok(MalData::Nil)
                                    }
                                }

                                Ok(_) =>
                                    tco_ast = then_form.clone(),

                                Err(err) =>
                                    return Err(err)
                            }

                            debug!("eval, if; tco_ast: {:?}; cont", tco_ast);
                            continue;
                        }

                        "fn*" => {
                            debug!("eval, > fn*");
                            let res = eval_fn(env.clone(), &list[1], &list[2]);
                            debug!("eval, < fn*");
                            return res;
                        }

                        "quote" => {
                            debug!("eval, quote; arg: {:?}", &list[1]);
                            return Ok((&list[1]).clone());
                        }

                        "quasiquote" => {
                            debug!("eval, quasiquote");
                            tco_ast = quasiquote(list.get(1).ok_or("form required")?)?;
                            continue;
                        }

                        "macroexpand" => {
                            let arg = &list[1];
                            debug!("macroexpand, arg: {:?}", arg);
                            return macroexpand(env.clone(), arg);
                        }
                        _ => (),
                    }
                }

                let eval_list = eval_ast(env.clone(), &tco_ast); 
                debug!("eval, eval_list: {:?}", eval_list);

                match eval_list {
                    Ok(MalData::List(ref l)) => {
                        match l.first() {
                            Some(&MalData::Function(ref f)) => {
                                debug!("(fun ...), f: {:?}", f);
                                return call_function(env, f, &l[1..]);
                            }

                            Some(&MalData::FnClosure(ref fnc)) => {
                                debug!("(funcl ...), fnc: {:?}", fnc);
                                tco_ast = *fnc.body.clone();

                                let fn_closure = fnc;
                                let parameters = &l[1..].to_owned();

                                debug!("apply_fn_closure, cl: {:?}, parameters: {:?}", fn_closure, parameters);

                                let outer_env = fn_closure.outer_env.clone();
                                let fn_env = Env::new(Some(outer_env), fn_closure.binds.as_slice(), parameters)?;
                                env = Rc::from(RefCell::from(fn_env));

                                continue;
                            }

                            Some(el) => {
                                let err_msg = format!("first element is not a function ({:?})", el);
                                return Err(EvalError::General(err_msg));
                            }

                            None =>
                                return Ok(MalData::Nil)
                        }

                    }

                    Ok(_) => return Ok(MalData::Nil),

                    Err(err) => {
                        return Err(err)
                    }
                }
            }

            // keine liste
            _ => {
                let eval_res = eval_ast(env, &tco_ast);
                debug!("eval, eval_res: {:?}", eval_res);
                return eval_res
            }
        }
    }
}

fn eval_ast(env: EnvType, ast: & MalData) -> Result<MalData, EvalError> {
    trace!("eval_ast");

    match ast {
        & MalData::Vector(ref vec) => {
            let mut eval_vec: Vec<MalData> = Vec::new();

            for el in vec.deref() {
                let res = eval(env.clone(), &el);

                if res.is_err() {
                    return res;
                } else {
                    eval_vec.push(res.unwrap());
                }
            }

            debug!("eval_ast, eval_vec: {:?}", eval_vec);

            Ok(MalData::Vector(Rc::new(eval_vec)))
        }

        & MalData::Map(ref map) => {
            let mut eval_map: HashMap<MapKey, MalData> = HashMap::new();

            let mut iter = map.into_iter();

            loop {
                match iter.next() {
                    None => break,

                    Some(( k, v )) => {
                        eval_map.insert(k.clone(), eval(env.clone(), &v).unwrap());
                    }
                }
            }

            debug!("eval_ast, eval_map: {:?}", eval_map);

            Ok(MalData::Map(eval_map))
        }

        & MalData::Symbol(ref sym) => {
            Env::get(&env, sym)
                .and_then( | v | Some(v.deref().clone()))
                .ok_or(EvalError::General(format!("'{}' not found", sym)))
        }

        & MalData::List(ref lst) => {
            let mut eval_list: Vec<MalData> = Vec::new();

            for el in lst.deref() {
                let res = eval(env.clone(), &el);

                if res.is_err() {
                    return res;
                } else {
                    eval_list.push(res.unwrap());
                }
            }

            debug!("eval_ast, eval_list: {:?}", eval_list);
            Ok(MalData::List(Rc::new(eval_list)))
        }

        _ => {
            debug!("eval_ast, ast: {:?}", ast);
            Ok(ast.clone())
        }
    }
}

fn print(input: &MalData) -> String {
    printer::pr_str(input, true)
}

fn env_insert_fun(env: EnvType, name: &str, fun: Rc<CallableFun>) {
        env.borrow_mut().set(&name.to_owned(), &MalData::Function(NativeFunction::new(name, fun)));
}


fn rep<'a, 'i>(repl_env: EnvType, input: &'i str) -> Result<String, EvalError> {
    let read_out = read(input)?;
    trace!("rep, read_out: {:?}", read_out);

    if let MalData::Nothing = read_out {
        return Ok("".to_owned());
    }

    debug!("rep, > eval");
    let res = eval(repl_env.clone(), &read_out).map(|r| print(&r));
    debug!("rep, < eval, res: {:?}", res);

    res
}

fn make_eval_closure(env_rc: EnvType) -> Rc<CallableFun> {
    let eval_closure: Rc<CallableFun> = Rc::from(move |fun_ctx: &FunContext, args: &[MalData]| { eval(env_rc.clone(), &args[0]).map_err(|e| format!("{}", e)) });

    eval_closure
}

// ordnet in der umgebung dem symbol 'eval' eine closure zu, die eval mit der uebergebenen umgebung (REPL-umgebung) und dem ersten
// parameter der closure aufruft.
//
// diese verrenkung mittels separater funktion schien noetig fuer ein erfolgreiches kompilieren. beim versuch, direkt in main
// eine entsprechende closure zu installieren verweigerte sich rust mit dem hinweis, dass die closure evtl. laenger als env_rc lebt.
fn insert_eval_closure(env_rc: Rc<RefCell<Env>>) {
    let dest_env = env_rc.clone();

    env_insert_fun(dest_env, "eval", make_eval_closure(env_rc.clone()));
}

fn main() {
    env_logger::init().unwrap();

    let repl_env = Env::new(None, &[], &[]).unwrap();
    let ns_map = init_ns_map();

    let env_rc = Rc::from(RefCell::from(repl_env));

    for (sym, fun) in ns_map {
        env_insert_fun(env_rc.clone(), sym, fun);
    }

    // 'eval' einer closure zuordnen, die eval mit REPL-env aufruft
    insert_eval_closure(env_rc.clone());

    // not
    rep(env_rc.clone(), "(def! not (fn* [a] (if a false true)))");

    // load-file
    rep(env_rc.clone(), "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \")\")))))");

    let empty_list = MalData::List(Rc::from(vec![]));

    if env::args().len() >= 2 {
        if env::args().len() > 2 {
            let argv_args: Vec<MalData> = env::args().skip(2).map(|a| MalData::String(a)).collect();
            let argv = MalData::List(Rc::from(argv_args.clone()));

            debug!("argv_args: {:?}, argv: {:?}", &argv_args, &argv);
            env_rc.borrow_mut().set(&"*ARGV*".to_owned(), &argv);
        } else {
            env_rc.borrow_mut().set(&"*ARGV*".to_owned(), &empty_list);
        }

        let path_arg = env::args().nth(1).unwrap();
        let path = Path::new(&path_arg);  // TODO existenz pruefen
        debug!(r#"(load-file "{}")"#, path.to_str().unwrap());

        let load_file_form = format!(r#"(load-file "{}")"#, path.to_str().unwrap());
        let res = rep(env_rc, &load_file_form);

        match res {
            Ok(result) =>
                println!("{}", result),

            Err(err) => 
                println!("error: {}", err)
        }

        return;
    } else {
        env_rc.borrow_mut().set(&"*ARGV*".to_owned(), &empty_list);
    }

    loop {
        let mut input = &mut String::new();

        debug!("main loop");

        print!("user> ");
        #[allow(unused_must_use)]
        io::stdout().flush();

        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        match rep(env_rc.clone(), &input) {
            Ok(ref e) if e.is_empty() => {}

            Ok(res) => println!("{}", res),

            Err(err) => println!("error: {}", err),
        }
    }
}

