use std::collections::HashMap;
use crate::ast::BindPoint;
use crate::string_pool::PoolS;

pub struct Scope {
    bindings: HashMap<PoolS, u64>
}

impl Scope {
    fn new() -> Self {
        Scope { bindings: HashMap::new() }
    }

    fn find(&self, name: PoolS) -> Option<u64> {
        self.bindings.get(&name).map(|x| *x)
    }

    fn add(&mut self, name: PoolS, id: u64) {
        self.bindings.insert(name, id);
    }
}

pub struct Scopes {
    scopes: Vec<Scope>
}

impl Scopes {
    pub fn new() -> Self {
        Scopes { scopes: Vec::new() }
    }

    pub fn push(&mut self) {
        self.scopes.push(Scope::new())
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn add(&mut self, name: PoolS, id: u64) {
        self.scopes.last_mut().map(|scope| scope.add(name, id));
    }

    pub fn find(&mut self, name: PoolS) -> BindPoint {
        for scope in &self.scopes {
            if let Some(id) = scope.find(name) {
                return BindPoint::BoundTo(id);
            }
        }
        BindPoint::Unbound(name)
    }
}





