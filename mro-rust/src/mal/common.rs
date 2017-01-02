use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use env::{Symbol, EnvType, Env};
use eval::EvalError;

pub trait MalFun: fmt::Debug {
    fn apply(&self, args: &[MalData]) -> Result<MalData, String>;
}

// fn eval(mut env: EnvType, ast: & MalData) -> Result<MalData, EvalError> {
pub type EvalFun = Fn(EnvType, &MalData) -> Result<MalData, EvalError>;

pub struct FunContext {
    pub eval: Option<Rc<CallableFun>>,
    pub eval2: Box<EvalFun>,
    pub env: Option<Rc<Env>>
}

pub type CallableFun = Fn(&FunContext, &[MalData]) -> Result<MalData, String>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapKey {
    True,
    False,
    String(String),
    Symbol(String),
    Keyword(String),
    Number(i32),
}

#[derive(Clone)]
pub struct FnClosure {
    pub outer_env: EnvType,
    pub binds: Vec<Symbol>,
    pub body: Box<MalData>,
    pub is_macro: bool
}

impl fmt::Debug for FnClosure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FnClosure {{ binds: {:?}, body: {:?} }}", self.binds, self.body)
    }
}


impl FnClosure {
    pub fn new(outer_env: EnvType, binds: &Vec<Symbol>, body: &MalData) -> FnClosure {
        FnClosure { outer_env: outer_env, binds: binds.clone(), body: Box::new(body.clone()), is_macro: false }
    }

    pub fn to_macro(&self) -> FnClosure {
        FnClosure { outer_env: self.outer_env.clone(), binds: self.binds.clone(), body: Box::new((*self.body).clone()), is_macro: true }
    }

    pub fn is_macro(&self) -> bool {
        self.is_macro
    }
}


// #[derive(Debug, Clone, PartialEq)]
#[derive(Debug, Clone)]
pub enum MalData {
    Nothing,
    Nil,
    True,
    False,
    String(String),
    Symbol(String),
    Keyword(String),
    Number(i32),
    List(Rc<Vec<MalData>>),
    Vector(Rc<Vec<MalData>>),
    Map(HashMap<MapKey, MalData>),
    Atom(Rc<RefCell<MalData>>),
    Function(NativeFunction),
    FnClosure(FnClosure),
    Exception(Box<MalData>)
}

impl PartialEq for MalData {
    fn eq(&self, other: &Self) -> bool {
        debug!("MalData::eq, self: {:?}, other: {:?}", self, other);

        match ( self, other ) {
            ( &MalData::Nothing, _ ) =>
                false,

            ( &MalData::Nil, &MalData::Nil ) =>
                true,

            ( &MalData::True, &MalData::True ) =>
                true,

            ( &MalData::False, &MalData::False ) =>
                true,

            ( &MalData::String(ref s1), &MalData::String(ref s2) ) =>
                s1 == s2,

            ( &MalData::Symbol(ref s1), &MalData::Symbol(ref s2) ) =>
                s1 == s2,

            ( &MalData::Keyword(ref kw1), &MalData::Keyword(ref kw2) ) => {
                let res = kw1 == kw2;
                debug!("eq, kw1: {:?}, kw2: {:?} -> {:?}", kw1, kw2, res);
                res
            }
                

            ( &MalData::Number(ref n1), &MalData::Number(ref n2) ) =>
                n1 == n2,

            ( ref v1, ref v2 ) if is_list_like(v1) && is_list_like(v2) => {
                let res = are_lists_equal(v1, v2).unwrap();
                debug!("eq, v1: {:?}, v2: {:?} -> {:?}", v1, v2, res);
                res
            }

            ( &MalData::Map(ref m1), &MalData::Map(ref m2) ) => {
                let res = m1 == m2;
                debug!("eq, m1: {:?}, m2: {:?} -> {:?}", m1, m2, res);
                res
            }

            ( &MalData::Atom(ref a1), &MalData::Atom(ref a2) ) =>
                a1 == a2,

            ( &MalData::Function(ref f1), &MalData::Function(ref f2) ) =>
                f1 == f2,

            ( &MalData::FnClosure(ref f1), &MalData::FnClosure(ref f2) ) =>
                f1 == f2,

            _ => {
                debug!("eq, default, self: {:?}, other: {:?} -> false", self, other);
                false
            }
        }
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl PartialEq for FnClosure {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum NativeFunctionSelector {
    Add,
    Sub,
    Mul,
    Div,
    Callable,
}


#[derive(Clone)]
pub struct NativeFunction {
    name: String,
    pub callable: Rc<CallableFun>
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "Function {{ name: {:?}, selector: {:?} }}", self.name, self.selector)
        write!(f, "Function {{ name: {:?} }}", self.name)
    }
    
}

impl NativeFunction {
    pub fn new(name: &str, callable: Rc<CallableFun>) -> NativeFunction {
        NativeFunction {
            name: name.to_owned(),
            callable: callable
        }
    }
}

pub fn is_list_like(value: &MalData) -> bool {
    match *value {
        MalData::List(_) |
        MalData::Vector(_) =>
            true,

        _ =>
            false
    }
}

pub fn are_lists_equal(l1: &MalData, l2: &MalData) -> Result<bool, String> {
    if !is_list_like(l1) || !is_list_like(l2) {
        return Err(format!("l1 und l2 muessen listenartig sein (l1: {:?}, l2: {:?})", l1, l2));
    }

    match ( l1, l2 ) {
        ( &MalData::List(ref l1), &MalData::List(ref l2) )
            | ( &MalData::List(ref l1), &MalData::Vector(ref l2) )
            | ( &MalData::Vector(ref l1), &MalData::List(ref l2) )
            | ( &MalData::Vector(ref l1), &MalData::Vector(ref l2) )
            if l1.len() == l2.len() => {
                let res = l1.iter().zip(l2.iter()).all( |( e1, e2 )| {
                    if is_list_like(e1) && is_list_like(e2) {
                        are_lists_equal(e1, e2).unwrap()
                    } else {
                        e1 == e2   
                    }
                });

                Ok(res)
            }

        _ =>
            Ok(false)
    }
}


pub fn make_mal_symbol(sym: &str) -> MalData {
    MalData::Symbol(sym.to_string())
}

pub fn make_mal_list_from_iter(iter: &mut Iterator<Item=&MalData>) -> MalData {
    let mut vec = Vec::new();
    vec.extend(iter.cloned());

    MalData::List(Rc::from(vec))
}

pub fn make_mal_list_from_vec(vec: Vec<MalData>) -> MalData {
    MalData::List(Rc::from(vec))
}

pub fn make_mal_list_from_slice(slice: &[MalData]) -> MalData {
    MalData::List(Rc::from(slice.to_vec()))
}

pub fn make_mal_vec_from_slice(slice: &[MalData]) -> MalData {
    MalData::Vector(Rc::from(slice.to_vec()))
}

pub fn is_mal_symbol(ast: &MalData) -> bool {
    if let &MalData::Symbol(_) = ast { true } else { false }
}

pub fn is_mal_list(ast: &MalData) -> bool {
    if let &MalData::List(_) = ast { true } else { false }
}

pub fn get_wrapped_list(ast: &MalData) -> Option<&Vec<MalData>> {
    match ast {
        &MalData::List(ref list) | &MalData::Vector(ref list) =>
            Some(list),

        _ =>
            None
    }
}

pub fn mal_symbol_name(ast: &MalData) -> Option<String> {
    if let &MalData::Symbol(ref sym) = ast {
        Some(sym.to_string())
    } else {
        None
    }
}

pub fn make_mal_keyword(kw: &str) -> MalData {
    MalData::Keyword(format!("\u{29e}{}", kw)) 
}

pub fn is_mal_keyword(value: &MalData) -> bool {
    if let &MalData::Keyword(_) = value { true } else { false }
}

pub fn is_mal_vector(value: &MalData) -> bool {
    if let &MalData::Vector(_) = value { true } else { false }
}

pub fn mal_bool_value(value: bool) -> MalData {
    if value { MalData::True } else { MalData::False }
}

pub fn is_mal_nil(value: &MalData) -> bool {
    if let &MalData::Nil = value { true } else { false }
}

pub fn is_mal_true(value: &MalData) -> bool {
    if let &MalData::True = value { true } else { false }
}

pub fn is_mal_false(value: &MalData) -> bool {
    if let &MalData::False = value { true } else { false }
}

pub fn is_mal_map(value: &MalData) -> bool {
    if let &MalData::Map(_) = value { true } else { false }
}

pub fn mapkey_for(value: &MalData) -> Result<MapKey, String> {
    match *value {
        MalData::True =>
            Ok(MapKey::True),

        MalData::False =>
            Ok(MapKey::False),

        MalData::String(ref string) =>
            Ok(MapKey::String(string.clone())),

        MalData::Symbol(ref string) =>
            Ok(MapKey::Symbol(string.to_string())),

        MalData::Keyword(ref string) =>
            Ok(MapKey::Keyword(string.clone())),

        MalData::Number(num) =>
            Ok(MapKey::Number(num)),

        _ =>
            Err(format!("mapkey_for, unhandled: {:?}", value))
    }
}

pub fn make_hashmap_from_kv_list(iter: &mut Iterator<Item=&MalData>) -> Result<HashMap<MapKey, MalData>, String> {
    let mut map: HashMap<MapKey, MalData> = HashMap::new();
    // let mut iter = kvs.iter();

    loop {
        let (k, v) = ( iter.next(), iter.next() );

        match (k, v) {
            (None, None) => break,

            (Some(k), None) => return Err(format!("value missing for map entry (key: {:?})", k)),

            (Some(k), Some(v)) => { map.insert(try!(mapkey_for(k)), v.clone()); }

            _ => break,
        }

        // debug!("TODO hashmap_from_kv_list, kvs: {:?}, k: {:?}, v: {:?}", kvs, k, v);
    }

    Ok(map)
}

pub fn make_mal_map_from_kv_list(iter: &mut Iterator<Item=&MalData>) -> Result<MalData, String> {
    Ok(MalData::Map(make_hashmap_from_kv_list(iter)?))
}
