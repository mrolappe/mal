extern crate mal;

#[macro_use]
extern crate log;
extern crate env_logger;
use log::LogLevel::Trace;

use std::io::{self, Write};
use std::collections::HashMap;
use std::ops::Deref;
use std::fmt;
use std::iter;
use std::slice::Iter;
use std::rc::Rc;
use std::cell::RefCell;

use mal::reader;
use mal::printer;
use mal::env::{EnvType, Env, Symbol};

use mal::common::MalData;
use mal::common::MapKey;
use mal::common::MalFun;
use mal::common::NativeFunction;
use mal::common::NativeFunctionSelector;
use mal::common::FnClosure;

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

fn call_function(f: & NativeFunction, args: & [MalData]) -> MalData {
    debug!("call_function, f: {:?}, args: {:?}", f, args);

    let result = f.apply(args);

    if log_enabled!(Trace) {
        for arg in args { trace!("arg: {:?}", arg) };
    }

    trace!("call_function, result: {:?}", result);
    result
}

fn call_fn_closure(f: & FnClosure, args: & [MalData]) -> MalData {
    debug!("call_fn_closure, f: {:?}, args: {:?}", f, args);
    let result = f.apply(args);

    if log_enabled!(Trace) {
        for arg in args { trace!("arg: {:?}", arg) };
    }

    trace!("call_fn_closure, result: {:?}", result);
    result
}

fn eval_do(env: EnvType, forms: & [MalData]) -> Result<MalData, EvalError> {
    let mut result = Ok(MalData::Nil);

    for form in forms {
        debug!("eval_do, form: {:?}", form);

        {
            let e = env.clone();
            result = eval_ast(e, &form);
            debug!("eval_do, result: {:?}", result);
        }
    }

    result
}

fn eval_if<'a: 'e, 'e, 'r>(env: EnvType, cond_form: & MalData, then_form: & MalData, else_form: Option<&'a MalData>) -> Result<MalData, EvalError> {
    let cond = eval(env.clone(), cond_form);
    match  cond {
        Ok(MalData::Nil)  | Ok(MalData::False)  =>
            if else_form.is_some() {
                eval(env.clone(), else_form.unwrap())
            } else {
                Ok(MalData::Nil)
            },

        Ok(_) =>
            eval(env, then_form),

        Err(err) =>
            Err(err)
    }
}

fn apply_fn_closure(fn_closure: &FnClosure, parameters: &[MalData]) -> Result<MalData, EvalError> {
    debug!("apply_fn_closure, cl: {:?}, parameters: {:?}", fn_closure, parameters);

    let outer_env = fn_closure.outer_env.clone();
    let fn_env = Env::new(Some(outer_env), fn_closure.binds.as_slice(), parameters)?;

    eval(Rc::new(RefCell::new(fn_env)), fn_closure.body.as_ref())
}

fn eval_fn(env: EnvType, binds: & MalData, body: & MalData) -> Result<MalData, EvalError> {
    debug!("eval_fn, env: {:?}, binds: {:?}, body: {:?}", env, binds, body);

    let binds_vec = if let &MalData::Vector(ref vec) = binds {
        let mut binds_vec = Vec::with_capacity(vec.len());

        for bind in vec.iter() {
            if let &MalData::Symbol(ref sym) = bind {
                binds_vec.push(sym.clone());
            } else {
                return Err(EvalError::General(format!("Expected symbol for bind, got: {:?}", bind)));
            }
        }

        binds_vec
    } else {
        return Err(EvalError::General("expected vector for binds".to_owned()));
    };

    Ok(MalData::FnClosure(FnClosure::new(env, &binds_vec, body)))
}

fn eval_let(env: EnvType, let_bindings: &[MalData], let_body: & MalData) -> Result<MalData, EvalError> {
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
        
        eval(let_env.clone(), let_body)
    }
}

fn eval_def(mut env: EnvType, name: Option<&MalData>, value: Option<&MalData>) -> Result<MalData, EvalError> {
    match ( name, value ) {
        ( None, None ) | ( None, Some(_) ) | ( Some(_), None ) =>
            Err(EvalError::General("def! requires a name and a value".to_owned())),

        ( Some(& MalData::Symbol(ref key)), Some(value) ) => {
            eval(env.clone(), value).map( | evaluated | {
                env.borrow_mut().set(&key, &evaluated); 
                debug!("def! {:?} -> {:?}/{:?}, env: {:?}", key, value, evaluated, env);
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
    match ast {
        & MalData::List(ref list) => {
            if list.is_empty() {
                return Ok(ast.clone())
            }

            // sonderformen/special forms
            if let Some(& MalData::Symbol(ref sym)) = list.first() {
                match sym.as_str() {
                    "def!" => {
                        debug!("eval, > def!, env: {:?}", env.clone());
                        let result = eval_def(env.clone(), list.get(1), list.get(2));
                        debug!("eval, < def!, env: {:?}", env.clone());
                        return result;
                    }

                    "let*" => {
                        debug!("eval, > let*, env: {:?}", env);
                        let let_body = list.get(2);

                        let let_bindings = match list.get(1) {
                            Some(&MalData::List(ref bindings)) | Some(&MalData::Vector(ref bindings)) =>
                                bindings,
                            _ =>
                                return Err(EvalError::General("let* bindings".to_string())) // TODO
                        };

                        let result = eval_let(env.clone(), let_bindings, let_body.unwrap());
                        debug!("eval, < let*, env: {:?}, result: {:?}", env.clone(), result);
                        return result;
                    }

                    "do" => {
                        debug!("eval, > do, env: {:?}", env);
                        return eval_do(env, &list[1..]);
                    }

                    "if" => {
                        debug!("eval, > if, env: {:?}", env);
                        return eval_if(env, &list[1], &list[2], list.get(3));
                    }

                    "fn*" => {
                        debug!("eval, > fn*, env: {:?}", env);
                        return eval_fn(env, &list[1], &list[2]);
                    }

                    _ => (),
                }
            }

            let eval_list = eval_ast(env, ast); 
            debug!("eval, eval_list: {:?}", eval_list);

            match eval_list {
                Ok(MalData::List(ref l)) => {
                    match l.first() {
                        Some(&MalData::Function(ref f)) => {
                            debug!("(fun ...), f: {:?}", f);
                            let res = call_function(f, &l[1..]);
                            return Ok(res)
                        }

                        Some(&MalData::FnClosure(ref fnc)) => {
                            debug!("(funcl ...), fnc: {:?}", fnc);
                            let res = try!(apply_fn_closure(fnc, &l[1..]));
                            return Ok(res)
                        }

                        Some(el) => {
                            let err_msg = format!("first element is not a function ({:?})", el);
                            Err(EvalError::General(err_msg))
                        }

                        None =>
                            Ok(MalData::Nil),
                    }

                }

                Ok(_) => Ok(MalData::Nil),

                Err(err) => {
                    Err(err)
                }
                
            }
        }

        _ => {
            let eval_res = eval_ast(env, ast);
            debug!("eval, eval_res: {:?}", eval_res);
            eval_res
        }
    }
}

fn eval_ast(env: EnvType, ast: & MalData) -> Result<MalData, EvalError> {
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

fn env_insert_native_fun<'e>(env: &'e mut Env,
                             name: &str,
                             selector: NativeFunctionSelector) {
        env.set(&name.to_owned(), &MalData::Function(NativeFunction::new(name, selector)));
}


fn rep<'a, 'i>(repl_env: EnvType, input: &'i str) -> Result<String, EvalError> {
    let read_out = read(input);
    trace!("rep, read_out: {:?}", read_out);

    if read_out.is_err() {
        return Err(EvalError::from(read_out.unwrap_err()));
    }

    debug!("rep, > eval; env: {:?}", repl_env.clone());
    let eval_out = eval(repl_env.clone(), &read_out?);
    debug!("rep, < eval; env: {:?}", repl_env.clone());

    if eval_out.is_ok() {
        Ok(print(&eval_out.unwrap()))
    } else {
        Err(eval_out.unwrap_err())
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut repl_env = Env::new(None, &[], &[]).unwrap();

    env_insert_native_fun(&mut repl_env, "+", NativeFunctionSelector::Add);
    env_insert_native_fun(&mut repl_env, "-", NativeFunctionSelector::Sub);
    env_insert_native_fun(&mut repl_env, "*", NativeFunctionSelector::Mul);
    env_insert_native_fun(&mut repl_env, "/", NativeFunctionSelector::Div);

    let env_rc = Rc::from(RefCell::from(repl_env));

    loop {
        let mut input = &mut String::new();

        debug!("main loop, env: {:?}", env_rc.clone());

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

