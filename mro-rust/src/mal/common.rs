use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;

use env::{Symbol, EnvType};

pub trait MalFun: fmt::Debug {
    fn apply(&self, args: &[MalData]) -> Result<MalData, String>;
}

pub type CallableFun = Fn(&[MalData]) -> Result<MalData, String>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapKey {
    True,
    False,
    String(String),
    Symbol(String),
    Keyword(String),
    Number(i32),
}

#[derive(Debug, Clone)]
pub struct FnClosure {
    pub outer_env: EnvType,
    pub binds: Vec<Symbol>,
    pub body: Box<MalData>
}

impl FnClosure {
    pub fn new(outer_env: EnvType, binds: &Vec<Symbol>, body: &MalData) -> FnClosure {
        FnClosure { outer_env: outer_env, binds: binds.clone(), body: Box::new(body.clone()) }
    }
}

// impl MalFun for FnClosure {
//     fn apply(&self, args: &[MalData]) -> Result<MalData, String> {
//         warn!("FIXME FnClosure.apply");

//         Ok(MalData::Nil)
//     }
// }
// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
    selector: NativeFunctionSelector,
    callable: Rc<CallableFun>
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Function {{ name: {:?}, selector: {:?} }}", self.name, self.selector)
    }
    
}

impl NativeFunction {
    pub fn new(name: &str, selector: NativeFunctionSelector, callable: Rc<CallableFun>) -> NativeFunction {
        NativeFunction {
            name: name.to_owned(),
            selector: selector,
            callable: callable
        }
    }
}

impl MalFun for NativeFunction {
    fn apply(&self, args: &[MalData]) -> Result<MalData, String> {
        // println!("call natfun {}, args: {:?}", self.name, args);

        if let NativeFunctionSelector::Callable = self.selector {
            return (self.callable)(args)
        }
        // let result = match self.selector {
        //     NativeFunctionSelector::Callable => {
        //         debug!("MalFun::apply, Callable");
        //         -1
        //     }
        //     NativeFunctionSelector::Add => number_arg(&args[0]) + number_arg(&args[1]),

        //     NativeFunctionSelector::Sub => number_arg(&args[0]) - number_arg(&args[1]),

        //     NativeFunctionSelector::Mul => number_arg(&args[0]) * number_arg(&args[1]),

        //     NativeFunctionSelector::Div => {
        //         number_arg(&args[0]).checked_div(number_arg(&args[1])).unwrap()
        //     }
        // };

        // Ok(MalData::Number(result))
        Ok(MalData::Number(666))
    }
}

