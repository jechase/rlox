use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
};

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
    pub fn define(&mut self, name: &Token, value: Value) {
        self.values.insert(name.lexeme.clone(), value);
    }

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
    where
        K: Hash + Eq,
        LoxStr: Borrow<K>,
    {
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
    }
}
