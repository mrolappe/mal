use regex::Regex;
use regex::Captures;

use common::MalData;


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

pub fn read_str(input: &str) -> Result<MalData, String> {
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


fn read_form<'r>(reader: &mut Reader) -> Result<MalData<'r>, String> {
    // FIXME lifetime
    // erstes token des readers untersuchen
    // unterscheidung nach erstem zeichen des tokens
    let result = match reader.peek() {
        // linke runde klammer -> read_list mit reader aufrufen
        Some("(") => {
            reader.next();
            read_list(reader)
        }

        // sonst read_atom mit reader aufrufen
        Some(_) => {
            if reader.peek().unwrap().starts_with(";") {
                reader.next();
                Ok(MalData::Nothing)    // TODO spezieller rueckgabewert eines speziellen rueckgabetyps?
            } else {
                let atom = read_atom(reader);
                atom.ok_or("failed to read atom".to_owned())
            }

        }

        None => Ok(MalData::Nothing),    // TODO
    };

    // rueckgabe: mal datentyp
    result
}


fn read_list<'r>(reader: &mut Reader) -> Result<MalData<'r>, String> {
    // FIXME lifetime
    let mut items = Vec::new();

    // read_form so lange mit reader aufrufen, bis zum auftreten eines ')'
    loop {
        match reader.peek() {
            // die ergebnisse werden in einer liste gesammelt
            Some(")") => {
                reader.next();
                return Ok(MalData::List(items));
            }

            Some(_) => {
                let form = read_form(reader);
                debug!("form: {:?}", &form);
                items.push(form.unwrap());
            }

            // tokens (vorzeitiges EOF ist fehler)
            None => {
                return Err("expected ')', got EOF".to_owned())
            }
        }
    }
}


fn read_atom<'r>(reader: &mut Reader) -> Option<MalData<'r>> {
    // FIXME lifetime
    let atom = reader.next();

    lazy_static!{
        static ref NUM_RE: Regex = Regex::new(r"^-?\d+$").unwrap();
    }

    // wert eines entsprechenden datentyps (z.b. ganzzahl oder symbol)
    // zurueckliefern anhand des token-inhalts
    match atom {
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
    }
}
