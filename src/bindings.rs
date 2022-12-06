use std::collections::HashMap;
use crate::ast::Type;
use crate::string_pool::PoolS;

pub struct Binding {
    pub output_name: String,
    pub typ: Type
}

impl Binding {
    pub fn new(output_name: String, typ: Type) -> Self {
        Binding { output_name, typ }
    }
}

pub struct Bindings {
    next: u64,
    map: HashMap<u64, Binding>,
    names: HashMap<PoolS, u64>
}

impl Bindings {
    pub fn new() -> Self {
        Bindings {
            next: 0,
            map: HashMap::new(),
            names: HashMap::new()
        }
    }

    fn grab_id(&mut self) -> u64 {
        let result = self.next;
        self.next += 1;
        result
    }

    pub fn new_binding(&mut self, name: PoolS, typ: Type) -> u64 {
        let num = *self.names.entry(name).and_modify(|x| *x += 1).or_insert(0);
        let output_name = format!("{}{}", name, num);

        let id = self.grab_id();
        self.map.insert(id, Binding::new(output_name, typ));

        id
    }

    pub fn get(&mut self, id: u64) -> &mut Binding {
        self.map.get_mut(&id).unwrap() // TODO: Determine if this unwrap is safe
    }
}