use std::io::{self, Write};

fn read(input: &str) -> &str {
    input
}


fn eval(input: &str) -> &str {
    input
}

fn print(input: &str) -> &str {
    input
}

fn rep(input: &str) -> &str {
    let read_out = read(input);
    let eval_out = eval(input);
    let print_out = print(input);

    print_out
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

        print!("{}", output);
    }
}
