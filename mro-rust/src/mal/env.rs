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
                ( Some(and), None ) if and == "&" => {
                    if let Some(rest_bind) = bi.next() {
                        env.set(rest_bind, &MalData::Nil);

                        break;
                    } else {
                        return Err("symbol expected for rest bind".to_owned())
                    }
                }

                ( Some(and), Some(rest_first) ) if and == "&" => {
                    if let Some(rest_bind) = bi.next() {
                        let mut rest_exprs: Vec<MalData> = Vec::new();

                        rest_exprs.push(rest_first.clone());

                        while let Some(expr) = ei.next() {
                            rest_exprs.push(expr.clone());
                        }

                        env.set(rest_bind, &MalData::List(Rc::from(rest_exprs)));

                        break;
                    } else {
                        return Err("symbol expected for rest bind".to_owned())
                    }
                }

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
        // debug!("env.set, data: {:?}, k: {:?}, v: {:?}", self.data, key, value);
        self.data.insert(key.clone(), Rc::from(value.clone()));
    }

    pub fn find(env: &EnvType, key: &EnvKey) -> Option<EnvType> {
        if env.borrow().data.contains_key(key) {
            // debug!("{:?} found in env {:?}", key, env);
            Some(env.clone())
        } else {
            let outer = &env.borrow().outer;

            let value = match outer {
                & Some(ref outer) => {
                    Env::find(&outer, key)
                }
                & None => None,
            };

            // debug!("{:?} found in outer ({:?}): {:?}", key, outer, value);
            value
        }
    }

    pub fn get<'e>(env: &'e EnvType, key: &EnvKey) -> Option<Rc<MalData>> {
        let val = match Env::find(env, key) {
            Some(env) =>
                Some(env.borrow().data.get(key).unwrap().clone()),
            None =>
                None
        };

        trace!("Env::get, key: {} -> {:?}", key, val);
        val
    }
}
