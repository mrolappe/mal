extern crate mal;

#[macro_use]
extern crate log;
extern crate env_logger;
use log::LogLevel::Trace;

use std::io::{self, Write};
use std::collections::HashMap;
use std::ops::Deref;
use std::fmt;

use mal::reader;
use mal::printer;
use mal::env::{Env};

use mal::common::MalData;
use mal::common::MapKey;
use mal::common::MalFun;
use mal::common::NativeFunction;
use mal::common::NativeFunctionSelector;

#[derive(Debug)]
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

fn read(input: &str) -> Result<MalData, String> {
    reader::read_str(input)
}

fn call_function(f: &NativeFunction, args: &[MalData]) -> MalData {
    let result = f.call(args);

    if log_enabled!(Trace) {
        for arg in args { trace!("arg: {:?}", arg) };
    }

    trace!("call_function, result: {:?}", result);
    result

}

fn eval<'a>(ast: &'a MalData, env: &'a mut Env) -> Result<MalData, EvalError> {
    // FIXME lifetime
    match ast {
        & MalData::List(ref list) => {
            if list.is_empty() {
                return Ok(ast.clone())
            }

            // sonderformen/special forms
            if let Some(& MalData::Symbol(ref sym)) = list.first() {
                match sym.as_str() {
                    "def!" => {
                        return match ( list.get(1), list.get(2) ) {
                            ( None, None ) | ( None, Some(_) ) | ( Some(_), None ) =>
                                Err(EvalError::General("def! requires a name and a value".to_owned())),

                            ( Some(& MalData::Symbol(ref key)), Some(value) ) => {
                                eval(value, env).map( |evaluated| { env.set(&key, &evaluated); evaluated })
                            }

                            ( key, value ) => {
                                let err_msg = format!("unhandled in def!, key: {:?}, value: {:?}", key, value);
                                Err(EvalError::General(err_msg.to_owned()))
                            }
                        }
                    }

                    "let*" => {
                        let let_env = &mut Env::new(Some(env));
                        let let_body = list.get(2);

                        let let_bindings = match list.get(1) {
                            Some(&MalData::List(ref bindings)) | Some(&MalData::Vector(ref bindings)) =>
                                bindings,
                            _ =>
                                return Err(EvalError::General("let* bindings".to_string())) // TODO
                        };

                        debug!("let body: {:?}", let_body);

                        let mut iter = let_bindings.iter();

                        loop {
                            match ( iter.next(), iter.next() ) {
                                ( Some(&MalData::Symbol(ref sym)), Some(def) ) => {
                                    let evaluated_def = eval(def, let_env);
                                    debug!("bind sym: {:?}, def: {:?} -eval-> {:?}", sym, def, evaluated_def);
                                    let_env.set(sym, &evaluated_def.unwrap());
                                }

                                ( None, None ) => break,

                                ( sym, def ) => {
                                    let err_msg = format!("error in let* binding; sym: {:?}, def: {:?}", sym, def);
                                    return Err(EvalError::General(err_msg))
                                }
                            }
                        }

                        return eval(let_body.unwrap(), let_env);
                    }

                    _ => (),
                }
            }

            let eval_list = eval_ast(ast, env);

            debug!("eval, eval_list: {:?}", eval_list);

            match eval_list {
                Ok(MalData::List(ref l)) => {
                    match l.first() {
                        Some(&MalData::Function(ref f)) => {
                            let res = call_function(&*f, &l[1..]);
                            return Ok(res)
                        }

                        _ => {
                            let err_msg = format!("first element is not a function ({:?})", l.first());
                            Err(EvalError::General(err_msg))
                        },
                    }

                }

                Ok(_) => Ok(MalData::Nil),

                Err(err) => {
                    Err(EvalError::General("???".to_owned()))
                }
                
            }
        }

        _ => {
            let eval_res = eval_ast(ast, env);
            debug!("eval, eval_res: {:?}", eval_res);
            eval_res
        }
    }
}

fn eval_ast<'a>(ast: &'a MalData, env: &'a mut Env) -> Result<MalData, EvalError> {
    // FIXME lifetime

    match *ast {
        MalData::Vector(ref vec) => {
            let mut eval_vec: Vec<MalData> = Vec::new();

            for el in vec {
                let res = eval(&el, env);

                if res.is_err() {
                    return res;
                } else {
                    eval_vec.push(res.unwrap());
                }
            }

            debug!("eval_ast, eval_vec: {:?}", eval_vec);

            Ok(MalData::Vector(eval_vec))
        }

        MalData::Map(ref map) => {
            let mut eval_map: HashMap<MapKey, MalData> = HashMap::new();

            let mut iter = map.into_iter();

            loop {
                match iter.next() {
                    None => break,

                    Some(( k, v )) => {
                        eval_map.insert(k.clone(), eval(&v, env).unwrap());
                    }
                }
            }

            debug!("eval_ast, eval_map: {:?}", eval_map);

            Ok(MalData::Map(eval_map))
            
        }

        MalData::Symbol(ref sym) => {
            env.get(sym)
                .and_then(|v| Some(v.clone()))
                .ok_or(EvalError::General(format!("'{}' not found", sym)))
        }

        MalData::List(ref lst) => {
            let mut eval_list: Vec<MalData> = Vec::new();

            for el in lst {
                let res = eval(&el, env);

                if res.is_err() {
                    return res;
                } else {
                    eval_list.push(res.unwrap());
                }
            }

            debug!("eval_ast, eval_list: {:?}", eval_list);
            Ok(MalData::List(eval_list))
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


fn rep(input: &str, repl_env: &mut Env) -> Result<String, EvalError> {
    let read_out = read(input);
    trace!("rep, read_out: {:?}", read_out);

    if read_out.is_err() {
        return Err(EvalError::from(read_out.unwrap_err()));
    }

    {
        let foo = &read_out.unwrap();
        let eval_out = eval(foo, repl_env);

        if eval_out.is_ok() {
            Ok(print(&eval_out.unwrap()))
        } else {
            Err(eval_out.unwrap_err())
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    info!("env_logger");

    let repl_env = &mut Env::new(None);

    env_insert_native_fun(repl_env, "+", NativeFunctionSelector::Add);
    env_insert_native_fun(repl_env, "-", NativeFunctionSelector::Sub);
    env_insert_native_fun(repl_env, "*", NativeFunctionSelector::Mul);
    env_insert_native_fun(repl_env, "/", NativeFunctionSelector::Div);

    loop {
        let mut input = String::new();

        print!("user> ");
        io::stdout().flush();

        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        match rep(&input, repl_env) {
            Ok(ref e) if e.is_empty() => {}

            Ok(res) => println!("{}", res),

            Err(err) => println!("error: {}", err),
        }
    }
}
