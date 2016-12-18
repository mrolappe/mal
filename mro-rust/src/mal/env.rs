use std::collections::HashMap;
use std::hash::Hash;

use common::MalData;

type EnvKey = String;
type Symbol = String;

#[derive(Debug)]
pub struct Env<'o, K=Symbol, V=MalData> where K: Eq+Hash {
    outer: Option<&'o Env<'o>>,
    data: HashMap<K, V>
}

impl<'o> Env<'o> {
    pub fn new(outer: Option<&'o Env<'o>>) -> Env {
        Env { outer: outer, data: HashMap::new() }
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
