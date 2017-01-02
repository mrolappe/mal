use regex::Regex;
use std::rc::Rc;

use common::{MalData, make_mal_list_from_vec, make_mal_symbol, make_mal_keyword, make_mal_map_from_kv_list};

struct Reader<'r> {
    tokens: Vec<&'r str>,
    index: usize,
}

type ReaderError = String;  // TODO eigener typ

impl<'r> Reader<'r> {
    pub fn new(tokens: &'r Vec<&'r str>) -> Reader<'r> {
        let filtered = tokens.iter().filter( |el| !el.starts_with(";")).map( |el| *el).collect::<Vec<&str>>();

        Reader {
            tokens: filtered,
            index: 0,
        }
    }

    fn next(&mut self) -> Option<&str> {
        // token an aktueller position zurueckliefern und position inkrementieren
        let result = Some(self.tokens[self.index]);
        self.index += 1;

        result
    }

    fn peek(&self) -> Option<&str> {
        if self.index < self.tokens.len() {
            // token an aktueller position zurueckliefern
            Some(self.tokens[self.index])    // TODO pruefung
        } else {
            None
        }
    }
}

pub fn read_str<'a>(input: &'a str) -> Result<MalData, String> {
    // tokenizer aufrufen
    let tokens = tokenizer(input);

    // neue instanz von reader erzeugen mit tokens
    let mut reader = Reader::new(&tokens);

    // read_form mit reader-instanz aufrufen
    read_form(&mut reader)
}


fn tokenizer(input: &str) -> Vec<&str> {
    lazy_static!{
        static ref RE: Regex = Regex::new(
            r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)"#).unwrap();

    }

    // for cap in RE.captures_iter(input) {
    //     println!("cap: {:?}", cap);
    // }

    let tokens = RE.captures_iter(input)
        .map(|cap| cap.at(1).unwrap())
        .collect();

    trace!("tokens: {:?}", tokens);

    tokens
}

fn read_form(reader: &mut Reader) -> Result<MalData, String> {
    trace!("read_form, peek: {:?}", reader.peek());

    // erstes token des readers untersuchen
    // unterscheidung nach erstem zeichen des tokens
    let result = match reader.peek() {
        // linke runde klammer -> read_list mit reader aufrufen
        Some("(") => {
            reader.next();
            read_list(reader, ")")
        }

        Some("[") => {
            reader.next();
            read_list(reader, "]")
        }

        Some("{") => {
            reader.next();
            read_list(reader, "}")
        }

        Some(")") | Some("]") | Some("}") => {
            Err(From::from("unbalanced parenthesis"))
        }

        Some("@") => {
            reader.next();
            let next_form = read_form(reader)?;
            let list = vec!(make_mal_symbol("deref"), next_form);

            Ok(MalData::List(Rc::from(list)))
        }

        // quote
        Some("'") => {
            reader.next();
            Ok(make_mal_list_from_vec(vec![ make_mal_symbol("quote"), read_form(reader)? ]))
        }

        // quasiquote
        Some("`") => {
            reader.next();
            Ok(make_mal_list_from_vec(vec![ make_mal_symbol("quasiquote"), read_form(reader)? ]))
        }

        // unquote
        Some("~") => {
            reader.next();
            Ok(make_mal_list_from_vec(vec![ make_mal_symbol("unquote"), read_form(reader)? ]))
        }

        // splice-unquote
        Some("~@") => {
            reader.next();
            Ok(make_mal_list_from_vec(vec![ make_mal_symbol("splice-unquote"), read_form(reader)? ]))
        }

        // sonst read_atom mit reader aufrufen
        Some(_) => {
            let atom = read_atom(reader);
            atom.ok_or("failed to read atom".to_owned())
        }

        None =>
            Ok(MalData::Nothing),    // TODO
    };

    // rueckgabe: mal datentyp
    result
}


fn read_list(reader: &mut Reader, delim: &str) -> Result<MalData, ReaderError> {
    let mut items = Vec::new();

    debug!("> read_list, delim: {}", delim);

    // read_form so lange mit reader aufrufen, bis zum auftreten eines ')'
    loop {
        // kommentare verschlucken
        while reader.peek().map_or(false, |p| p.starts_with(";")) {
            reader.next();
        }

        match (reader.peek(), delim) {
            // die ergebnisse werden in einer liste gesammelt
            (Some(")"), ")") => {
                reader.next();
                let list = MalData::List(Rc::new(items));
                debug!("< read_list, delim: {}, list: {:?}", delim, list);
                return Ok(list);
            }

            (Some("]"), "]") => {
                reader.next();
                let list = MalData::Vector(Rc::new(items));
                debug!("< read_list, delim: {}, list: {:?}", delim, list);
                return Ok(list);
            }

            (Some("}"), "}") => {
                reader.next();
                // let list = MalData::Map(make_hashmap_from_kv_list(&mut items.iter()).unwrap());
                let list = make_mal_map_from_kv_list(&mut items.iter())?;
                debug!("< read_list, delim: {}, list: {:?}", delim, list);
                return Ok(list);
            }

            (Some(_), delim) => {
                debug!("read_list, next: {:?}, delim: {:?}", reader.peek(), delim);
                let form = read_form(reader);
                debug!("read_list, delim: {:?}, form: {:?}", delim, &form);
                items.push(form?);
            }

            (None, delim) => {
                return Err(format!("expected '{}', got EOF", delim))
            }
        }
    }
}


fn read_atom(reader: &mut Reader) -> Option<MalData> {
    let atom = reader.next();

    lazy_static!{
        static ref NUM_RE: Regex = Regex::new(r"^-?\d+$").unwrap();
    }

    // wert eines entsprechenden datentyps (z.b. ganzzahl oder symbol)
    // zurueckliefern anhand des token-inhalts
    let res = match atom {
        Some(str) if str.starts_with("\"") && str.ends_with("\"") => {
            let str_content = &str[1..str.len() - 1];    // ohne die anfuehrungszeichen
            Some(MalData::String(transform_string(str_content)))
        }

        Some("nil") => {
            Some(MalData::Nil)
        }

        Some("true") => {
            Some(MalData::True) 
        }

        Some("false") => {
            Some(MalData::False) 
        }

        Some(e) if e.is_empty() => Some(MalData::Nothing),

        Some(num) if NUM_RE.is_match(num) => {
            debug!("read_atom, atom => {:?}", &atom);
            Some(MalData::Number(num.parse().ok().unwrap()))    // TODO fehlerbehandlung
        }

        Some(kw) if kw.starts_with(":") => {
            let name = kw.chars().skip(1).collect::<String>();
            Some(make_mal_keyword(name.as_str()))
        }

        Some(other) =>
            Some(make_mal_symbol(other.clone())),

        None => None
    };

    debug!("> read_atom, {:?} -> {:?}", atom, res);

    res
}

fn transform_string(string: &str) -> String {
    let newline_re = Regex::new(r#"\\n"#).unwrap();
    let dquote_re = Regex::new(r#"\\""#).unwrap();
    let backslash_re = Regex::new(r#"\\\\"#).unwrap();

    let mut res = newline_re.replace_all(string, "\n");
    res = dquote_re.replace_all(&res, r#"""#);
    res = backslash_re.replace_all(&res, r#"\"#);

    res
}
