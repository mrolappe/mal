use std::fmt;
use regex::Regex;
use regex::Captures;

use common::MalData;

// type IntFunction = Fn(&[MalData]) -> i32;

// impl fmt::Debug for MalData {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     write!(f, "Hi: ")
// }
// }

// impl Clone for MalData {
//     fn clone(&self) -> MalData {
//         match self {
//             &MalData::Symbol(s) => MalData::Symbol(s.clone()),

//             &MalData::Keyword(k) => MalData::Keyword(k.clone()),

//             &MalData::Number(n) => MalData::Number(n.clone()),

//             &MalData::List(l) => MalData::List(l.clone()),

//             &MalData::Function(f) => MalData::Function(f.clone())
//         } 
//     } 
// }



struct Reader<'r> {
    tokens: &'r Vec<&'r str>,
    index: usize,
}

impl<'r> Reader<'r> {
    pub fn new(tokens: &'r Vec<&'r str>) -> Reader<'r> {
        Reader {
            tokens: tokens,
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

pub fn read_str(input: &str) -> Option<MalData> {
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

    // println!("tokens: {:?}", tokens);

    tokens
}


fn read_form(reader: &mut Reader) -> Option<MalData<'static>> {  // FIXME lifetime
    // erstes token des readers untersuchen
    // unterscheidung nach erstem zeichen des tokens
    let result = match reader.peek() {
        // linke runde klammer -> read_list mit reader aufrufen
        Some("(") => {
            reader.next();
            read_list(reader)
        }

        Some(";") => {
            println!("kommentar");
            None    // TODO spezieller rueckgabewert eines speziellen rueckgabetyps?
        }

        // sonst read_atom mit reader aufrufen
        Some(_) => {
            let atom = read_atom(reader);
            atom
        }

        None => None,    // TODO
    };

    // rueckgabe: mal datentyp
    result
}


fn read_list(reader: &mut Reader) -> Option<MalData<'static>> {  // FIXME lifetime
    let mut items = Vec::new();

    // read_form so lange mit reader aufrufen, bis zum auftreten eines ')'
    loop {
        match reader.peek() {
            // die ergebnisse werden in einer liste gesammelt
            Some(")") => {
                return Some(MalData::List(items));
            }

            Some(_) => {
                let form = read_form(reader);
                // println!("form: {:?}", &form);
                items.push(form.unwrap());
            }

            // tokens (vorzeitiges EOF ist fehler)
            None => {
                println!("expected ')', got EOF");
                return None;    // TODO fehlerbehandlung}
            }
        }
    }
}


fn read_atom(reader: &mut Reader) -> Option<MalData<'static>> {  // FIXME lifetime
    let atom = reader.next().unwrap();

    lazy_static!{
        static ref NUM_RE: Regex = Regex::new(r"^\d+$").unwrap();
    }

    // wert eines entsprechenden datentyps (z.b. ganzzahl oder symbol)
    // zurueckliefern anhand des token-inhalts
    if NUM_RE.is_match(atom) {
        // println!("read_atom, atom => {:?}", &atom);
        Some(MalData::Number(atom.parse().ok().unwrap()))    // TODO fehlerbehandlung 
    } else if atom.chars().next().unwrap() == ':' {
        Some(MalData::Keyword('\u{29e}'.to_string() + atom))
    }
    else {
        Some(MalData::Symbol(atom.to_string()))
    }
}
