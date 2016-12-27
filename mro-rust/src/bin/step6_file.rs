extern crate mal;

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
use std::borrow::Borrow;

use mal::reader;
use mal::printer;
use mal::env::{EnvType, Env, Symbol};

use mal::common::MalData;
use mal::common::MapKey;
use mal::common::MalFun;
use mal::common::NativeFunction;
use mal::common::NativeFunctionSelector;
use mal::common::{FnClosure, CallableFun};

use mal::core::init_ns_map;

#[derive(Debug, Clone)]
enum EvalError {
    General(String)
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

fn call_function(f: & NativeFunction, args: & [MalData]) -> Result<MalData, EvalError> {
    debug!("call_function, f: {:?}, args: {:?}", f, args);

    let result = f.apply(args);

    if log_enabled!(Trace) {
        for arg in args { trace!("arg: {:?}", arg) };
    }

    trace!("call_function, result: {:?}", result);
    result.map_err( | e | EvalError::from(e))
}

// fn eval_do(env: EnvType, forms: & [MalData]) -> Result<MalData, EvalError> {
// }

// fn eval_if<'a: 'e, 'e, 'r>(env: EnvType, cond_form: & MalData, then_form: & MalData, else_form: Option<&'a MalData>) -> Result<MalData, EvalError> {
// }

fn apply_fn_closure(fn_closure: &FnClosure, parameters: &[MalData]) -> Result<MalData, EvalError> {
    debug!("apply_fn_closure, cl: {:?}, parameters: {:?}", fn_closure, parameters);

    let outer_env = fn_closure.outer_env.clone();
    let fn_env = Env::new(Some(outer_env), fn_closure.binds.as_slice(), parameters)?;

    eval(Rc::new(RefCell::new(fn_env)), fn_closure.body.as_ref())
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

            Ok(MalData::FnClosure(FnClosure::new(env, &binds_vec, body)))
        }

        _ =>
            Err(EvalError::General("expected vector for binds".to_owned()))
    }
}

// fn eval_let(env: EnvType, let_bindings: &[MalData], let_body: & MalData) -> Result<MalData, EvalError> {
// }

fn eval_def(mut env: EnvType, name: Option<&MalData>, value: Option<&MalData>) -> Result<MalData, EvalError> {
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

fn eval(mut env: EnvType, ast: & MalData) -> Result<MalData, EvalError> {
    let mut tco_ast: MalData = ast.clone();

    loop {
        let let_body: &MalData;
        // let mut tco_ast = tco_ast;

        debug!("eval, loop; tco_ast: {:?}", tco_ast);

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

                    "let*" => {
                        debug!("eval, > let*");
                        let_body = list.get(2).unwrap();

                        let let_bindings = match list.get(1) {
                            Some(&MalData::List(ref bindings)) | Some(&MalData::Vector(ref bindings)) =>
                                bindings,
                            _ =>
                                return Err(EvalError::General("let* bindings".to_string())) // TODO
                        };

                        // let result = eval_let(env.clone(), let_bindings, let_body.unwrap());
                        debug!("let body: {:?}", let_body);

                        let mut iter = let_bindings.iter();

                        {
                            // TODO binds, exprs
                            let let_env = Rc::new(RefCell::new(Env::new(None, &[], &[]).unwrap()));

                            loop {
                                match ( iter.next(), iter.next() ) {
                                    ( Some(&MalData::Symbol(ref sym)), Some(ref def) ) => {
                                        let evaluated_def = eval(let_env.clone(), &def.clone());

                                        debug!("bind sym: {:?}, def: {:?} -eval-> {:?}", sym, def, evaluated_def);

                                        let_env.borrow_mut().set(&sym.clone(), &evaluated_def.unwrap());
                                    }

                                    ( None, None ) => break,

                                    ( sym, def ) => {
                                        let err_msg = format!("error in let* binding; sym: {:?}, def: {:?}", sym, def);
                                        return Err(EvalError::General(err_msg))
                                    }
                                }
                            }

                            // eval(let_env.clone(), let_body)
                            env = let_env;
                            tco_ast = let_body.clone();
                            continue;
                        }

                        // debug!("eval, < let*, result: {:?}", result);
                        // return result;
                    }

                    "do" => {
                        debug!("eval, > do");
                        let mut result = Ok(MalData::Nil);
                        let forms = &list[1..list.len() - 1];

                        for form in forms {
                            debug!("eval_do, form: {:?}", form);

                            {
                                let e = env.clone();
                                result = eval_ast(e, &form);
                                debug!("eval_do, result: {:?}", result);
                            }
                        }

                        tco_ast = list.last().unwrap().clone();  // TODO fehlerbehandlung
                        continue;
                        // return eval_do(env, &list[1..]);
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
                        // return eval_if(env, &list[1], &list[2], list.get(3));
                    }

                    "fn*" => {
                        debug!("eval, > fn*");
                        let res = eval_fn(env.clone(), &list[1], &list[2]);
                        debug!("eval, < fn*");
                        return res;
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
                            return call_function(f, &l[1..]);
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
                            // let res = try!(apply_fn_closure(fnc, &l[1..]));
                            // return Ok(res)
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
    debug!("eval_ast");

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

fn env_insert_fun(env: & mut Env, name: &str, fun: Rc<CallableFun>) {
        env.set(&name.to_owned(), &MalData::Function(NativeFunction::new(name, NativeFunctionSelector::Callable, fun)));
}


fn rep<'a, 'i>(repl_env: EnvType, input: &'i str) -> Result<String, EvalError> {
    let read_out = read(input);
    trace!("rep, read_out: {:?}", read_out);

    if read_out.is_err() {
        return Err(EvalError::from(read_out.unwrap_err()));
    }

    debug!("rep, > eval");
    let eval_out = eval(repl_env.clone(), &read_out?);
    debug!("rep, < eval");

    if eval_out.is_ok() {
        Ok(print(&eval_out.unwrap()))
    } else {
        Err(eval_out.unwrap_err())
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut repl_env = Env::new(None, &[], &[]).unwrap();

    let ns_map = init_ns_map();

    for (sym, fun) in ns_map {
        env_insert_fun(&mut repl_env, sym, fun);
    }

    let env_rc = Rc::from(RefCell::from(repl_env));

    rep(env_rc.clone(), "(def! not (fn* [a] (if a false true)))");

    loop {
        let mut input = &mut String::new();

        debug!("main loop");

        print!("user> ");
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

