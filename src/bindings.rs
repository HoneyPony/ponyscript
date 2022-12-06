use std::collections::HashMap;
use crate::ast::Type;
use crate::string_pool::PoolS;

#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct VarID(u64);

#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct FunID(u64);

pub struct VarBinding {
    pub output_name: String,
    pub typ: Type
}

impl VarBinding {
    pub fn new(output_name: String, typ: Type) -> Self {
        VarBinding { output_name, typ }
    }
}

pub struct FunBinding {
    pub output_name: String,
    pub return_type: Type,
    pub args: Vec<VarID>
}

impl FunBinding {
    pub fn new(output_name: String, return_type: Type, args: Vec<VarID>) -> Self {
        FunBinding { output_name, return_type, args }
    }
}

pub struct Bindings {
    next: u64,
    var_map: HashMap<VarID, VarBinding>,
    fun_map: HashMap<FunID, FunBinding>,
    names: HashMap<PoolS, u64>
}

impl Bindings {
    pub fn new() -> Self {
        Bindings {
            next: 0,
            var_map: HashMap::new(),
            fun_map: HashMap::new(),
            names: HashMap::new()
        }
    }

    fn grab_id(&mut self) -> u64 {
        let result = self.next;
        self.next += 1;
        result
    }

    pub fn new_var_binding(&mut self, name: PoolS, typ: Type) -> VarID {
        let num = *self.names.entry(name).and_modify(|x| *x += 1).or_insert(0);
        let output_name = format!("{}{}", name, num);

        let id = VarID(self.grab_id());
        self.var_map.insert(id, VarBinding::new(output_name, typ));

        id
    }

    pub fn get_var(&mut self, id: VarID) -> &mut VarBinding {
        self.var_map.get_mut(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    pub fn new_fun_binding(&mut self, name: PoolS, return_type: Type, args: Vec<VarID>) -> FunID {
        // TODO: Possibly support functions with different arguments. Actually, we probably
        // definitely want to do that...
        let output_name = format!("{}_f", name);

        let id = FunID(self.grab_id());
        self.fun_map.insert(id, FunBinding::new(output_name, return_type, args));

        id
    }

    pub fn get_fun(&mut self, id: FunID) -> &mut FunBinding {
        self.fun_map.get_mut(&id).unwrap() // TODO: Determine if this unwrap is safe
    }
}