use itertools::join;

use common::MalData;
use common::MapKey;

pub trait PrStr {
    fn pr_str(&self, print_readably: bool) -> String;
}

impl PrStr for MapKey {
    fn pr_str(&self, print_readably: bool) -> String {
        match *self {
            // MalData::Nil => "nil".to_owned(),
            MapKey::True => "true".to_owned(),
            MapKey::False => "false".to_owned(),
            MapKey::String(ref string) => make_readable_string(string),
            MapKey::Symbol(ref sym) => sym.clone(),
            MapKey::Keyword(ref kw) => kw.chars().skip(1).collect(),
            MapKey::Number(ref num) => num.to_string(),
        }
    }
}
impl<'d> PrStr for MalData<'d> {
    fn pr_str(&self, print_readably: bool) -> String {
        match *self {
            MalData::Nothing => "".to_owned(),
            MalData::Nil => "nil".to_owned(),
            MalData::True => "true".to_owned(),
            MalData::False => "true".to_owned(),
            MalData::String(ref string) => make_readable_string(string),
            MalData::Symbol(ref sym) => sym.clone(),  // TODO symbolname
            MalData::Keyword(ref kw) => kw.chars().skip(1).collect(),
            MalData::Number(ref num) => num.to_string(),  // TODO zahl

            MalData::List(ref elements) => {
                let mut out = String::from("(");

                out.push_str(join(elements.iter().map(|e| pr_str(e, print_readably)), " ")
                    .as_str());
                out.push_str(")");

                out
            }

            MalData::Vector(ref elements) => {
                let mut out = String::from("[");

                out.push_str(join(elements.iter().map(|e| pr_str(e, print_readably)), " ")
                    .as_str());
                out.push_str("]");

                out
            }

            MalData::Map(ref elements) => {
                let mut out = String::from("{");

                out.push_str(join(elements.iter().map(|(k, v)| {
                                      format!("{} {}",
                                              pr_str(k, print_readably),
                                              pr_str(v, print_readably))
                                  }),
                                  " ")
                    .as_str());
                out.push_str("}");

                out
            }

            MalData::Function(_) => "fun".to_string(),
        }
    }
}
pub fn pr_str(data: &PrStr, print_readably: bool) -> String {
    data.pr_str(print_readably)
}

fn make_readable_string(string: &String) -> String {
    // TODO
    format!("{}", string)
}
