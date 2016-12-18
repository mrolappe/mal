use std::fmt;
use std::collections::HashMap;

pub trait MalFun: fmt::Debug {
    fn call(&self, args: &[MalData]) -> MalData;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapKey {
    True,
    False,
    String(String),
    Symbol(String),
    Keyword(String),
    Number(i32),
}

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
    List(Vec<MalData>),
    Vector(Vec<MalData>),
    Map(HashMap<MapKey, MalData>),
    Function(NativeFunction),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum NativeFunctionSelector {
    Add,
    Sub,
    Mul,
    Div,
}


#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct NativeFunction {
    name: String,
    selector: NativeFunctionSelector,
}

impl NativeFunction {
    pub fn new(name: &str, selector: NativeFunctionSelector) -> NativeFunction {
        NativeFunction {
            name: name.to_owned(),
            selector: selector,
        }
    }
}

impl MalFun for NativeFunction {
    fn call(&self, args: &[MalData]) -> MalData {
        // println!("call natfun {}, args: {:?}", self.name, args);

        let result = match self.selector {
            NativeFunctionSelector::Add => number_arg(&args[0]) + number_arg(&args[1]),

            NativeFunctionSelector::Sub => number_arg(&args[0]) - number_arg(&args[1]),

            NativeFunctionSelector::Mul => number_arg(&args[0]) * number_arg(&args[1]),

            NativeFunctionSelector::Div => {
                number_arg(&args[0]).checked_div(number_arg(&args[1])).unwrap()
            }
        };

        MalData::Number(result)
    }
}

fn number_arg(arg: &MalData) -> i32 {
    debug!("number_arg, arg: {:?}", arg);

    let number = match *arg {
        MalData::Number(num) => num,

        _ => {
            panic!("arg ist keine zahl");
        }// FIXME fehlerbehandlung
    };

    number
}
