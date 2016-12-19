use std::collections::HashMap;
use std::hash::Hash;
use std::slice::Iter;

use common::MalData;

type EnvKey = String;
type Symbol = String;

#[derive(Debug)]
pub struct Env<'o, K=Symbol, V=MalData> where K: Eq+Hash {
    outer: Option<&'o Env<'o>>,
    data: HashMap<K, V>
}

fn sym_name(data: &MalData) -> Option<String> {
    if let &MalData::Symbol(ref sym) = data {
        Some(sym.clone())
    } else {
        None
    }
}
impl<'o> Env<'o> {
    pub fn new(outer: Option<&'o Env<'o>>, mut binds: Iter<MalData>, mut exprs: Iter<MalData>) -> Result<Env<'o>, String> {
        let mut env = Env { outer: outer, data: HashMap::new() };

        loop {
            match ( binds.next(), exprs.next() ) {
                ( Some(bind), Some(expr) ) =>
                    env.set(sym_name(&bind).as_ref().unwrap(), expr),

                ( None, None) =>
                    break,

                ( bind, expr) =>
                    return Err(format!("illegal binding: {:?}, {:?}", bind, expr))
            }
        };

        Ok(env)
    }

    pub fn set(&mut self, key: &EnvKey, value: &MalData) -> () {
        debug!("env.set, data: {:?}, k: {:?}, v: {:?}", self.data, key, value);
        self.data.insert(key.clone(), value.clone());
    }

    pub fn find(&self, key: &EnvKey) -> Option<&Env> {
        if self.data.contains_key(key) {
            debug!("{:?} found in env {:?}", key, self);
            Some(self)
        } else {
            let value = match self.outer {
                Some(outer) => outer.find(key),
                None => None,
            };

            debug!("{:?} found in outer ({:?}): {:?}", key, self.outer, value);
            value
        }
    }

    pub fn get(&self, key: &EnvKey) -> Option<&MalData> {
        match self.find(key) {
            Some(env) =>
                env.data.get(key),
            None =>
                None
        }
    }
}
