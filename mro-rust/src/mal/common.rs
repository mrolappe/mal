use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use env::{Symbol, EnvType, Env};

pub trait MalFun: fmt::Debug {
    fn apply(&self, args: &[MalData]) -> Result<MalData, String>;
}

pub struct FunContext {
    pub eval: Option<Rc<CallableFun>>,
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
    pub body: Box<MalData>
}

impl fmt::Debug for FnClosure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FnClosure {{ binds: {:?}, body: {:?} }}", self.binds, self.body)
    }
}


impl FnClosure {
    pub fn new(outer_env: EnvType, binds: &Vec<Symbol>, body: &MalData) -> FnClosure {
        FnClosure { outer_env: outer_env, binds: binds.clone(), body: Box::new(body.clone()) }
    }
}


#[derive(Debug, Clone, PartialEq)]
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
    FnClosure(FnClosure)
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


