use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    mem::replace,
    rc::Rc,
};

use crate::*;

#[derive(Default, Debug)]
pub struct Scope {
    values:    HashMap<LoxStr, Value>,
    enclosing: Option<ScopeHandle>,
}

type ScopeHandle = Rc<RefCell<Scope>>;

impl Scope {
    pub fn get<K>(&self, name: &K) -> Option<Value>
    where
        K: Hash + Eq + ?Sized,
        LoxStr: Borrow<K>,
    {
        self.values.get(name).map(Clone::clone).or_else(|| {
            self.enclosing.as_ref().and_then(|enclosing| RefCell::borrow(enclosing).get(name))
        })
    }

    pub fn assign<K>(&mut self, name: &K, value: Value) -> Option<Value>
    where
        K: Hash + Eq + ?Sized,
        LoxStr: Borrow<K>,
    {
        if self.values.contains_key(name) {
            return Some(replace(self.values.get_mut(name).unwrap(), value));
        }

        self.enclosing.as_ref().and_then(|enclosing| enclosing.borrow_mut().assign(name, value))
    }

    pub fn define<K>(&mut self, name: K, value: Value)
    where
        K: Into<LoxStr>,
    {
        self.values.insert(name.into(), value);
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    global: ScopeHandle,
    scope:  ScopeHandle,
}

impl Environment {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Scope::default()));
        let scope = global.clone();
        Environment {
            global,
            scope,
        }
    }

    pub fn with_enclosing(parent: &Environment) -> Self {
        Environment {
            global: parent.global.clone(),
            scope:  Rc::new(RefCell::new(Scope {
                values:    Default::default(),
                enclosing: Some(parent.scope.clone()),
            })),
        }
    }

    pub fn define<S>(&mut self, name: S, value: Value)
    where
        S: Into<LoxStr>,
    {
        self.scope.borrow_mut().define(name, value)
    }

    pub fn assign<K>(&mut self, name: &K, value: Value) -> Option<Value>
    where
        K: Hash + Eq + ?Sized,
        LoxStr: Borrow<K>,
    {
        self.scope.borrow_mut().assign(name, value)
    }

    pub fn get<K>(&self, name: &K) -> Option<Value>
    where
        K: Hash + Eq + ?Sized,
        LoxStr: Borrow<K>,
    {
        RefCell::borrow(&self.scope).get(name)
    }
    pub fn define_global<S>(&self, name: S, value: Value)
    where
        S: Into<LoxStr>,
    {
        self.global.borrow_mut().define(name, value)
    }

    pub fn assign_global<K>(&mut self, name: &K, value: Value) -> Option<Value>
    where
        K: Hash + Eq + ?Sized,
        LoxStr: Borrow<K>,
    {
        self.global.borrow_mut().assign(name, value)
    }

    pub fn get_global<K>(&self, name: &K) -> Option<Value>
    where
        K: Hash + Eq + ?Sized,
        LoxStr: Borrow<K>,
    {
        RefCell::borrow(&self.global).get(name)
    }

    pub fn get_enclosing(&self) -> Option<Environment> {
        RefCell::borrow(&self.scope).enclosing.as_ref().map(|enclosing| Environment {
            global: self.global.clone(),
            scope:  enclosing.clone(),
        })
    }

    pub fn ancestor(&self, depth: usize) -> Option<Environment> {
        let mut ret = Some(self.clone());
        for _ in 0..depth {
            ret = ret.and_then(|ret| ret.get_enclosing());
        }
        ret
    }

    pub fn get_at<K, I>(&self, name: &K, depth: I) -> Option<Value>
    where
        K: Hash + Eq + ?Sized,
        LoxStr: Borrow<K>,
        I: Into<Option<usize>>,
    {
        if let Some(depth) = depth.into() {
            self.ancestor(depth).and_then(|e| e.get(name))
        } else {
            self.get_global(name)
        }
    }
}
