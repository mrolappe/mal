extern crate mal;

use std::io::{self, Write};
use std::collections::HashMap;
use std::ops::Deref;

use mal::reader;
use mal::printer;

use mal::common::MalData;
use mal::common::MalFun;
use mal::common::NativeFunction;
use mal::common::NativeFunctionSelector;

fn read(input: &str) -> Option<MalData> {
    reader::read_str(input)
}

type IntFunction = fn(&[MalData]) -> i32;

fn call_function<'d>(f: &'d MalFun, args: &[MalData]) -> MalData<'d> {  // FIXME lifetime
    let result = f.call::<'d>(args);

    for arg in args { println!("arg: {:?}", arg) };

    println!("result: {:?}", result);
    result
        
}

fn call_custom_function(args: &[MalData]) -> MalData<'static> { // FIXME lifetime
    println!("custom func: {:?}", args.get(0));

    for arg in args { println!("arg: {:?}", arg) };

    // println!("result: {}", result);
    MalData::Number(0)
    
}

type Environment = HashMap<Box<String>, Box<MalFun>>;

fn eval<'a>(ast: &'a MalData, env: &'a Environment) -> MalData<'a> { // FIXME lifetime
    match *ast {
        MalData::List(ref list) =>
            if list.is_empty() {
                ast.clone()
            } else {
                let eval_list = eval_ast(ast, env);

                match eval_list {
                    MalData::List(l) => {
                        match l.first() {
                            Some(&MalData::Function(f)) => {
                                call_function(f, &l[1..])
                            },

                            _ => panic!("FIXME fehlerbehandlung; erstes element ist keine funktion")
                        }
                    }

                    _ => panic!("Scheiss die Wand an!, {:?}", eval_list)
                }
            },

        _ => eval_ast(ast, env)
    }
}

fn eval_ast<'a>(ast: &'a MalData, env: &'a Environment) -> MalData<'a> { // FIXME lifetime
    // TODO Result als rueckgabetyp

    match *ast {
        MalData::Symbol(ref sym) => {
            // FIXME fehlermeldung, wenn symbol nicht gebunden

            let val = env.get(sym).unwrap();
            MalData::Function(val.deref())
            
        }

        MalData::List(ref l) => {
            let mut eval_list : Vec<MalData> = Vec::new();

            for e in l {
                eval_list.push(eval(&e, env))
            }

            MalData::List(eval_list)
            
        },

        _ => ast.clone()
    }
}

fn print(input: &MalData) -> String {
    printer::pr_str(input)
}

fn env_insert_native_fun<'e>(env: &'e mut Environment, name: &str, selector: NativeFunctionSelector) {
    env.insert(Box::new(name.to_owned()), Box::new(NativeFunction::new(name, selector))); 
}
fn rep(input: &str) -> String {
    let repl_env: &mut Environment = &mut HashMap::new();

    env_insert_native_fun(repl_env, "+", NativeFunctionSelector::Add);
    env_insert_native_fun(repl_env, "-", NativeFunctionSelector::Sub);
    env_insert_native_fun(repl_env, "*", NativeFunctionSelector::Mul);
    env_insert_native_fun(repl_env, "/", NativeFunctionSelector::Div);  // TODO fehlerbehandlug

    if let Some(read_out) = read(input) {
        let eval_out = eval(&read_out, &repl_env);
        let print_out = print(&eval_out);
        print_out
    } else {
        "err".to_string()    // TODO erweiterte fehlerbehandlung
    }

}

fn main() {
    loop {
        let mut input = String::new();

        print!("user> ");
        io::stdout().flush();

        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let output = rep(&input);

        println!("{}", output);
    }
}
