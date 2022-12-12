use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::iter::zip;
use crate::ast::{Node, Type};
use crate::string_pool::PoolS;

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct VarID(u64);

#[derive(Debug)]
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
    reverse_fun_map: HashMap<PoolS, Vec<(FunID, Vec<VarID>)>>,
    names: HashMap<PoolS, u64>
}

impl Bindings {
    pub fn new() -> Self {
        Bindings {
            next: 0,
            var_map: HashMap::new(),
            fun_map: HashMap::new(),
            reverse_fun_map: HashMap::new(),
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
        let output_name = format!("{}_{}", name, num);

        let id = VarID(self.grab_id());
        self.var_map.insert(id, VarBinding::new(output_name, typ));

        id
    }

    pub fn get_var(&self, id: VarID) -> &VarBinding {
        self.var_map.get(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    pub fn get_var_mut(&mut self, id: VarID) -> &mut VarBinding {
        self.var_map.get_mut(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    pub fn new_fun_binding(&mut self, name: PoolS, return_type: Type, args: Vec<VarID>) -> Result<FunID, String> {
        let existing = self.find_fun_from_vars(name, &args);

        if existing.is_some() {
            return Err(format!("function {} already defined with these arguments", name));
        }

        // TODO: Possibly support functions with different arguments. Actually, we probably
        // definitely want to do that...
        let mut output_name = format!("{}_", name);
        for arg in &args {
            let typ = &self.get_var(*arg).typ;
            output_name += &typ.to_string();
            output_name.push('_');
        }

        let id = FunID(self.grab_id());

        let list = self.reverse_fun_map.entry(name).or_insert(vec![]);
        list.push((id, args.clone()));

        self.fun_map.insert(id, FunBinding::new(output_name, return_type, args));

        Ok(id)
    }

    pub fn find_fun_from_vars(&self, name: PoolS, args: &Vec<VarID>) -> Option<FunID> {
        let options = self.reverse_fun_map.get(&name)?;

        for option in options {
            if &option.1 == args {
                return Some(option.0)
            }
        }

        None
    }

    pub fn find_fun_from_compat_nodes(&self, name: PoolS, args: &Vec<Node>) -> Option<FunID> {
        let options = self.reverse_fun_map.get(&name)?;

        for option in options {
            if option.1.len() != args.len() { continue; }

            let matches_all = zip(&option.1, args).map(|pair| {
                let left = &self.get_var(*pair.0).typ;
                let right = &pair.1.get_expr_type(self);

                left.eq_or_may_coerce(right)
            }).filter(|entry| *entry == false).count() == 0;

            if matches_all {
                return Some(option.0);
            }
        }

        None
    }

    pub fn get_fun(&self, id: FunID) -> &FunBinding {
        self.fun_map.get(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    pub fn get_fun_mut(&mut self, id: FunID) -> &mut FunBinding {
        self.fun_map.get_mut(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    pub fn fun_bindings(&self) -> Values<'_, FunID, FunBinding> {
        self.fun_map.values()
    }

}