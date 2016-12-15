extern crate mal;

#[macro_use]
extern crate log;
extern crate env_logger;
use log::LogLevel::Trace;

use std::io::{self, Write};
use std::collections::HashMap;
use std::ops::Deref;

use mal::reader;
use mal::printer;

use mal::common::MalData;
use mal::common::MapKey;
use mal::common::MalFun;
use mal::common::NativeFunction;
use mal::common::NativeFunctionSelector;

fn read(input: &str) -> Result<MalData, String> {
    reader::read_str(input)
}

type IntFunction = fn(&[MalData]) -> i32;

fn call_function<'d>(f: &'d MalFun, args: &[MalData]) -> MalData<'d> {
    // FIXME lifetime
    let result = f.call(args);

    if log_enabled!(Trace) {
        for arg in args { trace!("arg: {:?}", arg) };
    }

    trace!("call_function, result: {:?}", result);
    result

}

type Environment = HashMap<Box<String>, Box<MalFun>>;

fn eval<'a>(ast: &'a MalData, env: &'a Environment) -> Result<MalData<'a>, String> {
    // FIXME lifetime
    match *ast {
        MalData::List(ref list) => {
            if list.is_empty() {
                Ok(ast.clone())
            } else {
                let eval_list = eval_ast(ast, env);
                if eval_list.is_err() {
                    return eval_list;
                }
                debug!("eval, eval_list: {:?}", eval_list);

                match eval_list.unwrap() {
                    MalData::List(l) => {
                        match l.first() {
                            Some(&MalData::Function(f)) => {
                                let res = call_function(f, &l[1..]);
                                Ok(res)
                            }

                            _ => Err(format!("first element is not a function ({:?})", l.first())),
                        }
                    }

                    foo => panic!("Scheiss die Wand an!, {:?}", foo),
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

fn eval_ast<'a>(ast: &'a MalData, env: &'a Environment) -> Result<MalData<'a>, String> {
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
                .ok_or(format!("'{}' not found", sym))
                .and_then(|v| Ok(MalData::Function(v.deref())))
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

fn env_insert_native_fun<'e>(env: &'e mut Environment,
                             name: &str,
                             selector: NativeFunctionSelector) {
    env.insert(Box::new(name.to_owned()),
               Box::new(NativeFunction::new(name, selector)));
}
fn rep(input: &str) -> Result<String, String> {
    let repl_env: &mut Environment = &mut HashMap::new();

    env_insert_native_fun(repl_env, "+", NativeFunctionSelector::Add);
    env_insert_native_fun(repl_env, "-", NativeFunctionSelector::Sub);
    env_insert_native_fun(repl_env, "*", NativeFunctionSelector::Mul);
    env_insert_native_fun(repl_env, "/", NativeFunctionSelector::Div);  // TODO fehlerbehandlug

    let read_out = read(input);
    trace!("rep, read_out: {:?}", read_out);

    if read_out.is_err() {
        return Err(read_out.unwrap_err());
    }

    {
        let foo = &read_out.unwrap();
        let eval_out = eval(foo, &repl_env);

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

    loop {
        let mut input = String::new();

        print!("user> ");
        io::stdout().flush();

        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        match rep(&input) {
            Ok(ref e) if e.is_empty() => {}

            Ok(res) => println!("{}", res),

            Err(err) => println!("error: {}", err),
        }
    }
}
