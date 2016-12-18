extern crate mal;

use std::io::{self, Write};

use mal::reader;
use mal::printer;

use mal::common::MalData;

fn read(input: &str) -> Option<MalData> {
    reader::read_str(input).ok()
}


fn eval(input: MalData) -> MalData {
   input 
}

fn print(input: &MalData, print_readably: bool) -> String {
    printer::pr_str(input, print_readably)
}

fn rep(input: &str) -> String {
    if let Some(read_out) = read(input) {
        let eval_out = eval(read_out);
        let print_out = print(&eval_out, true);
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
