use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::cell::RefCell;
use common::MalData;

pub type Symbol = String;
type EnvKey = Symbol;
pub type EnvType = Rc<RefCell<Env>>;


#[derive(Debug, Clone)]
pub struct Env<K=Symbol, V=Rc<MalData>> where K: Eq+Hash {
    outer: Option<EnvType>,
    data: HashMap<K, V>
}

impl Env {
    pub fn new(outer: Option<EnvType>, binds: &[Symbol], exprs: &[MalData]) -> Result<Env, String> {
        let mut env = Env { outer: outer, data: HashMap::new() };

        let mut bi = binds.iter();
        let mut ei = exprs.iter();

        loop {
            match ( bi.next(), ei.next() ) {
                ( Some(bind), Some(expr) ) => {
                    debug!("Env::new, bind {:?} -> {:?}", bind, expr);
                    env.set(&bind.clone(), expr);
                }

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
        self.data.insert(key.clone(), Rc::from(value.clone()));
    }

    pub fn find(env: &EnvType, key: &EnvKey) -> Option<EnvType> {
        if env.borrow().data.contains_key(key) {
            debug!("{:?} found in env {:?}", key, env);
            Some(env.clone())
        } else {
            let outer = &env.borrow().outer;

            let value = match outer {
                & Some(ref outer) => {
                    Env::find(&outer, key)
                }
                & None => None,
            };

            debug!("{:?} found in outer ({:?}): {:?}", key, outer, value);
            value
        }
    }

    pub fn get<'e>(env: &'e EnvType, key: &EnvKey) -> Option<Rc<MalData>> {
        match Env::find(env, key) {
            Some(env) =>
                Some(env.borrow().data.get(key).unwrap().clone()),
            None =>
                None
        }
    }
}
