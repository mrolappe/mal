use itertools::join;

use common::MalData;

pub fn pr_str(data: &MalData) -> String {
    match *data {
        MalData::Symbol(ref sym) => sym.clone(),  // TODO symbolname
        MalData::Keyword(ref kw) => kw.chars().skip(1).collect(),
        MalData::Number(ref num) => num.to_string(),  // TODO zahl
        MalData::List(ref elements) => {
            let mut out = String::from("(");

            out.push_str(join(elements.iter().map(|e| pr_str(e)), " ").as_str());
            out.push_str(")");

            out
        },

        MalData::Function(f) => "fun".to_string(),
    }
}
