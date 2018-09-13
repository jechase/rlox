use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
};

<<<<<<< HEAD
use crate::*;

#[derive(Default, Debug)]
pub struct Environment {
    values:    HashMap<LoxStr, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn child(enclosing: Environment) -> Environment {
        Environment {
            enclosing: Some(Box::new(enclosing)),
            ..Default::default()
        }
    }
=======
use crate::{
    token::Token,
    value::Value,
    LoxStr,
};

#[derive(Default)]
pub struct Environment {
    values: HashMap<LoxStr, Value>,
}

impl Environment {
>>>>>>> 1d35dea... to_owned -> into_owned
    pub fn define(&mut self, name: &Token, value: Value) {
        self.values.insert(name.lexeme.clone(), value);
    }

<<<<<<< HEAD
    pub fn assign(&mut self, name: &Token, value: Value) {
        self.get_mut(&name.lexeme).map(|opt| *opt = value);
    }

    pub fn get<K>(&self, name: &K) -> Option<&Value>
    where
        K: Hash + Eq,
        LoxStr: Borrow<K>,
    {
        if let Some(v) = self.values.get(name) {
            return Some(v);
        }

        if let Some(parent) = self.enclosing.as_ref() {
            return parent.get(name);
        }

        None
    }

    fn get_mut<K>(&mut self, name: &K) -> Option<&mut Value>
=======
    pub fn get<K>(&mut self, name: &K) -> Option<&Value>
>>>>>>> 1d35dea... to_owned -> into_owned
    where
        K: Hash + Eq,
        LoxStr: Borrow<K>,
    {
<<<<<<< HEAD
        if let Some(v) = self.values.get_mut(name) {
            return Some(v);
        }

        if let Some(parent) = self.enclosing.as_mut() {
            return parent.get_mut(name);
        }

        None
    }

    pub fn into_parent(self) -> Option<Environment> {
        self.enclosing.map(|e| *e)
=======
        self.values.get(name)
>>>>>>> 1d35dea... to_owned -> into_owned
    }
}
