use itertools::join;
use regex::Regex;

use common::MalData;
use common::MapKey;

pub trait PrStr {
    fn pr_str(&self, print_readably: bool) -> String;
}

impl PrStr for MapKey {
    fn pr_str(&self, print_readably: bool) -> String {
        if print_readably {
        }

        match *self {
            // MalData::Nil => "nil".to_owned(),
            MapKey::True => "true".to_owned(),
            MapKey::False => "false".to_owned(),
            MapKey::String(ref string) => make_readable_string(string),
            MapKey::Symbol(ref sym) => sym.clone(),
            MapKey::Keyword(ref kw) => ":".chars().chain(kw.chars().skip(1)).collect(),
            MapKey::Number(ref num) => num.to_string(),
        }
    }
}

impl<'d> PrStr for MalData {
    fn pr_str(&self, print_readably: bool) -> String {
        match *self {
            MalData::Nothing => "".to_owned(),
            MalData::Atom(ref atom) => format!("(atom {})", atom.borrow().pr_str(print_readably)),
            MalData::Nil => "nil".to_owned(),
            MalData::True => "true".to_owned(),
            MalData::False => "false".to_owned(),
            MalData::String(ref string) => if print_readably { make_readable_string(string) } else { string.clone() },
            MalData::Symbol(ref sym) => sym.clone(),  // TODO symbolname
            MalData::Keyword(ref kw) => ":".chars().chain(kw.chars().skip(1)).collect(),
            MalData::Number(ref num) => num.to_string(),  // TODO zahl

            MalData::List(ref elements, _) => {
                let mut out = String::from("(");

                out.push_str(join(elements.iter().map(|e| pr_str(e, print_readably)), " ")
                    .as_str());
                out.push_str(")");

                out
            }

            MalData::Vector(ref elements, _) => {
                let mut out = String::from("[");

                out.push_str(join(elements.iter().map(|e| pr_str(e, print_readably)), " ")
                    .as_str());
                out.push_str("]");

                out
            }

            MalData::Map(ref elements, _) => {
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

            MalData::Function(_) | MalData::FnClosure(_) => "#<function>".to_string(),

            MalData::Exception(_) =>
                "#<exception>".to_string(),
        }
    }
}

pub fn pr_str(data: &PrStr, print_readably: bool) -> String {
    data.pr_str(print_readably)
}

fn make_readable_string(string: &String) -> String {
    let newline_re = Regex::new(r"\n").unwrap();
    let dquote_re = Regex::new(r#"""#).unwrap();
    let backslash_re = Regex::new(r#"\\"#).unwrap();

    let mut escaped = backslash_re.replace_all(string, r#"\\"#);
    escaped = newline_re.replace_all(&escaped, r#"\n"#);
    escaped = dquote_re.replace_all(&escaped, r#"\""#);

    let mut res = String::from("\"");
    res.push_str(&escaped);
    res.push_str("\"");

    // println!("mrs: {} -> {}", string, res);
    res.to_owned()
}
