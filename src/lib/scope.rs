use std::{
    collections::HashMap,
    hash::Hash,
    mem::replace,
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
        let parent = if let Some(scope) =
            parent.and_then(|idx| self.scopes.get_mut(idx))
        {
            scope.children += 1;
            parent
        } else {
            None
        };

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
