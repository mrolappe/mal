use itertools::join;

use common::MalData;

pub fn pr_str(data: &MalData, print_readably: bool) -> String {
    match *data {
        MalData::Nothing => "".to_owned(),
        MalData::Nil => "nil".to_owned(),
        MalData::True => "#t".to_owned(),
        MalData::False => "#f".to_owned(),
        MalData::String(ref string) => make_readable_string(string),
        MalData::Symbol(ref sym) => sym.clone(),  // TODO symbolname
        MalData::Keyword(ref kw) => kw.chars().skip(1).collect(),
        MalData::Number(ref num) => num.to_string(),  // TODO zahl
        MalData::List(ref elements) => {
            let mut out = String::from("(");

            out.push_str(join(elements.iter().map(|e| pr_str(e, print_readably)), " ").as_str());
            out.push_str(")");

            out
        },

        MalData::Function(f) => "fun".to_string(),
    }
}

fn make_readable_string(string: &String) -> String {
    format!("TODO: make_readable: {}", string)
}
