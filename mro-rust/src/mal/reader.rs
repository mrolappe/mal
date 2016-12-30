use std::collections::HashMap;
use regex::Regex;
use std::rc::Rc;
use common::MalData;
use common::MapKey;

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
    debug!("read_form, peek: {:?}", reader.peek());

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
            let list = vec!(MalData::Symbol("deref".to_owned()), next_form);

            Ok(MalData::List(Rc::from(list)))
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
        while reader.peek().unwrap().starts_with(";") {
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
                let list = MalData::Map(hashmap_from_kv_list(items).unwrap());
                debug!("< read_list, delim: {}, list: {:?}", delim, list);
                return Ok(list);
            }

            (Some(_), delim) => {
                debug!("read_list, next: {:?}, delim: {:?}", reader.peek(), delim);
                let form = read_form(reader);
                debug!("read_list, delim: {:?}, form: {:?}", delim, &form);
                items.push(form.unwrap());
            }

            (None, delim) => {
                return Err(format!("expected '{}', got EOF", delim))
            }
        }
    }
}

fn mapkey_for(atom: &MalData) -> Result<MapKey, ReaderError> {
    match *atom {
        MalData::True =>
            Ok(MapKey::True),

        MalData::False =>
            Ok(MapKey::False),

        MalData::String(ref string) =>
            Ok(MapKey::String(string.clone())),

        MalData::Symbol(ref string) =>
            Ok(MapKey::Symbol(string.clone())),

        MalData::Keyword(ref string) =>
            Ok(MapKey::Keyword(string.clone())),

        MalData::Number(num) =>
            Ok(MapKey::Number(num)),

        _ =>
            Err(format!("mapkey_for, unhandled: {:?}", atom))
    }
}


fn hashmap_from_kv_list(kvs: Vec<MalData>) -> Result<HashMap<MapKey, MalData>, ReaderError> {
    let mut map: HashMap<MapKey, MalData> = HashMap::new();
    let mut iter = kvs.iter();

    loop {
        let (k, v) = ( iter.next(), iter.next() );

        match (k, v) {
            (None, None) => break,

            (Some(k), None) => return Err(format!("value missing for map entry (key: {:?})", k)),

            (Some(k), Some(v)) => { map.insert(try!(mapkey_for(k)), v.clone()); }

            _ => break,
        }

        debug!("TODO hashmap_from_kv_list, kvs: {:?}, k: {:?}, v: {:?}", kvs, k, v);
    }

    Ok(map)
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
            Some(MalData::Keyword('\u{29e}'.to_string() + kw))
        }

        Some(other) => Some(MalData::Symbol(other.to_string())),

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
