use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    fmt,
    hash::Hash,
    rc::Rc,
};

use crate::*;

#[derive(Debug, Clone)]
pub struct LoxInstance {
    inner: InstanceHandle,
}

impl PartialEq<LoxInstance> for LoxInstance {
    fn eq(&self, right: &LoxInstance) -> bool {
        Rc::ptr_eq(&self.inner, &right.inner)
    }
}

type InstanceHandle = Rc<RefCell<InstanceInner>>;

#[derive(Debug)]
struct InstanceInner {
    class:  Rc<LoxClass>,
    fields: HashMap<LoxStr, Value>,
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<instance {}>", RefCell::borrow(&self.inner).class.name)
    }
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        LoxInstance {
            inner: Rc::new(RefCell::new(InstanceInner {
                class,
                fields: Default::default(),
            })),
        }
    }

    pub fn get<K>(&self, name: &K) -> Option<Value>
    where
        K: Eq + Hash,
        LoxStr: Borrow<K>,
    {
        let borrowed = RefCell::borrow(&self.inner);
        borrowed.fields.get(name).cloned().or_else(|| {
            Some(Value::LoxFn(borrowed.class.find_method(self.clone(), name)?))
        })
    }

    pub fn set<K>(&self, name: K, value: Value) -> Option<Value>
    where
        K: Into<LoxStr>,
    {
        self.inner.borrow_mut().fields.insert(name.into(), value)
    }
}
