use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    hash::Hash,
    mem::replace,
    rc::Rc,
};

use generational_arena::Arena;

pub use generational_arena::Index;

use crate::{
    value::Value,
    LoxStr,
};

#[derive(Debug)]
pub struct ScopeMgr {
    global: Scope,
    scopes: Arena<Scope>,
}

impl Default for ScopeMgr {
    fn default() -> Self {
        ScopeMgr::new()
    }
}

#[derive(Default, Debug)]
pub struct Scope {
    values:    HashMap<LoxStr, Value>,
    enclosing: Option<Index>,
    children:  usize,
}

impl ScopeMgr {
    pub fn new() -> ScopeMgr {
        ScopeMgr {
            global: Scope::default(),
            scopes: Arena::new(),
        }
    }
    pub fn create_scope(&mut self, parent: Option<Index>) -> Index {
        if let Some(parent) = parent {
            self.add_ref(parent);
        }

        let new_scope = Scope {
            enclosing: parent,
            values:    Default::default(),
            children:  1,
        };

        self.scopes.insert(new_scope)
    }

    pub fn destroy_scope(&mut self, index: Index) -> Option<Index> {
        if self.del_ref(index) == 0 {
            self.scopes.remove(index).and_then(|old| {
                old.enclosing.map(|parent| self.destroy_scope(parent));
                old.enclosing
            })
        } else {
            self.scopes.get(index).and_then(|scope| scope.enclosing)
        }
    }

    pub fn get<K>(&self, scope: Option<Index>, name: &K) -> Option<&Value>
    where
        K: Hash + Eq + AsRef<str>,
    {
        let scope = if let Some(scope) = scope {
            scope
        } else {
            return self.global.values.get(name.as_ref().as_bytes());
        };

        let scope = &self.get_scope(scope);

        scope
            .values
            .get(name.as_ref().as_bytes())
            .or_else(|| self.get(scope.enclosing, name))
    }

    pub fn define<S>(&mut self, scope: Option<Index>, name: S, value: Value)
    where
        S: Into<LoxStr>,
    {
        let scope = if let Some(scope) = scope {
            scope
        } else {
            self.global.values.insert(name.into(), value);
            return;
        };

        self.get_scope_mut(scope).values.insert(name.into(), value);
    }

    pub fn assign<K>(
        &mut self,
        scope: Option<Index>,
        name: &K,
        value: Value,
    ) -> Option<Value>
    where
        K: Hash + Eq + AsRef<str>,
    {
        let entry: Option<&mut Value> = self.get_mut(scope, name);
        entry.map(|v| replace(v, value))
    }

    fn get_mut<K>(
        &mut self,
        scope: Option<Index>,
        name: &K,
    ) -> Option<&mut Value>
    where
        K: Hash + Eq + AsRef<str>,
    {
        if scope.is_none() {
            return self.global.values.get_mut(name.as_ref().as_bytes());
        }

        let scope = scope.unwrap();

        if self.contains(Some(scope), name) {
            return self
                .get_scope_mut(scope)
                .values
                .get_mut(name.as_ref().as_bytes());
        }

        self.get_mut(self.get_scope(scope).enclosing, name)
    }

    fn contains<K>(&self, scope: Option<Index>, name: &K) -> bool
    where
        K: Hash + Eq + AsRef<str>,
    {
        match scope {
            None => &self.global,
            Some(idx) => self.get_scope(idx),
        }
        .values
        .contains_key(name.as_ref().as_bytes())
    }

    fn get_scope(&self, scope: Index) -> &Scope {
        &self.scopes[scope]
    }

    fn get_scope_mut(&mut self, scope: Index) -> &mut Scope {
        &mut self.scopes[scope]
    }

    pub fn add_ref(&mut self, scope: Index) -> usize {
        let scope = self.get_scope_mut(scope);
        scope.children += 1;
        scope.children
    }

    pub fn del_ref(&mut self, scope: Index) -> usize {
        let scope = self.get_scope_mut(scope);
        scope.children -= 1;
        scope.children
    }
}

pub struct Environment {
    mgr:   Rc<RefCell<ScopeMgr>>,
    scope: Option<Index>,
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(scope) = self.scope {
            let mut env = f.debug_struct("Scope");
            let mgr = self.mgr.borrow();
            let scope = mgr.get_scope(scope);
            env.field("values", &scope.values);
            env.field("enclosing", &self.get_enclosing());
            env
        } else {
            let mut env = f.debug_struct("Global");
            env.field("values", &self.mgr.borrow().global.values);
            env
        }.finish()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            mgr:   Rc::new(RefCell::new(ScopeMgr::default())),
            scope: None,
        }
    }

    pub fn with_enclosing(parent: &Environment) -> Self {
        let mgr = parent.mgr.clone();
        let scope = {
            let mut mgr = mgr.borrow_mut();
            Some(mgr.create_scope(parent.scope))
        };

        Environment {
            mgr,
            scope,
        }
    }

    pub fn define<S>(&mut self, name: S, value: Value)
    where
        S: Into<LoxStr>,
    {
        self.mgr.borrow_mut().define(self.scope, name, value)
    }

    pub fn assign<K>(&mut self, name: &K, value: Value) -> Option<Value>
    where
        K: Hash + Eq + AsRef<str>,
    {
        self.mgr.borrow_mut().assign(self.scope, name, value)
    }

    pub fn get<K>(&self, name: &K) -> Option<Value>
    where
        K: Hash + Eq + AsRef<str>,
    {
        self.mgr.borrow().get(self.scope, name).cloned()
    }
    pub fn define_global<S>(&mut self, name: S, value: Value)
    where
        S: Into<LoxStr>,
    {
        self.mgr.borrow_mut().define(None, name, value)
    }

    pub fn assign_global<K>(&mut self, name: &K, value: Value) -> Option<Value>
    where
        K: Hash + Eq + AsRef<str>,
    {
        self.mgr.borrow_mut().assign(None, name, value)
    }

    pub fn get_global<K>(&self, name: &K) -> Option<Value>
    where
        K: Hash + Eq + AsRef<str>,
    {
        self.mgr.borrow().get(None, name).cloned()
    }

    pub fn get_enclosing(&self) -> Option<Environment> {
        self.scope.map(|scope| {
            let enclosing = self.mgr.borrow().get_scope(scope).enclosing;
            Environment {
                mgr:   self.mgr.clone(),
                scope: enclosing,
            }
        })
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        let mut mgr = self.mgr.borrow_mut();
        if let Some(scope) = self.scope {
            mgr.destroy_scope(scope);
        }
    }
}

impl Clone for Environment {
    fn clone(&self) -> Environment {
        if let Some(scope) = self.scope {
            self.mgr.borrow_mut().add_ref(scope);
        }

        Environment {
            mgr:   self.mgr.clone(),
            scope: self.scope.clone(),
        }
    }
}
