use std::collections::HashMap;
use crate::ast::BindPoint;
use crate::bindings::{FunID, VarID};
use crate::string_pool::PoolS;

pub struct Scope {
    var_bindings: HashMap<PoolS, VarID>,

    #[allow(unused)]
    fun_bindings: HashMap<PoolS, FunID>
}

impl Scope {
    fn new() -> Self {
        Scope { var_bindings: HashMap::new(), fun_bindings: HashMap::new() }
    }

    fn find_var(&self, name: PoolS) -> Option<VarID> {
        self.var_bindings.get(&name).map(|x| *x)
    }

    fn add_var(&mut self, name: PoolS, id: VarID) {
        self.var_bindings.insert(name, id);
    }

    #[allow(unused)]
    fn find_fun(&self, name: PoolS) -> Option<FunID> {
        self.fun_bindings.get(&name).map(|x| *x)
    }

    #[allow(unused)]
    fn add_fun(&mut self, name: PoolS, id: FunID) {
        self.fun_bindings.insert(name, id);
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

    pub fn add_var(&mut self, name: PoolS, id: VarID) {
        self.scopes.last_mut().map(|scope| scope.add_var(name, id));
    }

    pub fn find_var(&mut self, name: PoolS) -> BindPoint<VarID> {
        for scope in &self.scopes {
            if let Some(id) = scope.find_var(name) {
                return BindPoint::BoundTo(id);
            }
        }
        BindPoint::Unbound(name)
    }
}





