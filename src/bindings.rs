use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::zip;
use crate::ast::{GetExprType, Node, TypeName};
use crate::string_pool::PoolS;

pub trait GetID<T> {
    fn get_id(&self) -> Option<T>;
}

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct VarID(u64);

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct FunID(u64);

impl GetID<Self> for VarID {
    fn get_id(&self) -> Option<Self> {
        Some(*self)
    }
}

impl GetID<Self> for FunID {
    fn get_id(&self) -> Option<Self> {
        Some(*self)
    }
}

#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
#[allow(unused)]
pub enum Namespace {
    Global,
    StaticCall(PoolS),
    DynamicCall(PoolS)
}

impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Namespace::Global => f.write_str("g_"),
            Namespace::StaticCall(typ) => f.write_fmt(format_args!("s{}_", typ)),
            Namespace::DynamicCall(typ) => f.write_fmt(format_args!("d{}_", typ))
        }
    }
}

pub struct VarBinding {
    pub output_name: String,
    pub typ: TypeName,
    pub is_member: bool
}

impl VarBinding {
    pub fn new(output_name: String, typ: TypeName) -> Self {
        VarBinding { output_name, typ, is_member: false }
    }
}

pub struct FunBinding {
    pub output_name: String,
    pub return_type: TypeName,
    pub args: Vec<VarID>,
    pub called_on: Option<TypeName>,
    pub namespace: Namespace
}

impl FunBinding {
    pub fn new(namespace: Namespace, output_name: String, return_type: TypeName, args: Vec<VarID>) -> Self {
        FunBinding { output_name, return_type, args, called_on: None, namespace }
    }
}

pub struct Type {
    pub members: Vec<VarID>, // Is this what we want...?
    pub base_class: Option<TypeName>,
    pub output_name: String,

    // TODO: Move to much nicer system for all this...
    pub associated_typename: TypeName
}

impl Type {
    pub fn new(typename: &TypeName) -> Type {
        return Type {
            members: Vec::new(),
            base_class: None,
            output_name: typename.to_string(),
            associated_typename: typename.clone()
        }
    }
}

pub struct Bindings {
    next: u64,
    var_map: HashMap<VarID, VarBinding>,
    fun_map: HashMap<FunID, FunBinding>,
    reverse_fun_map: HashMap<(Namespace, PoolS), Vec<(FunID, Vec<VarID>)>>,
    names: HashMap<PoolS, u64>,

    type_map: HashMap<TypeName, Type>
}

impl Bindings {
    pub fn new() -> Self {
        Bindings {
            next: 0,
            var_map: HashMap::new(),
            fun_map: HashMap::new(),
            reverse_fun_map: HashMap::new(),
            names: HashMap::new(),

            type_map: HashMap::new()
        }
    }

    fn grab_id(&mut self) -> u64 {
        let result = self.next;
        self.next += 1;
        result
    }

    pub fn new_var_binding(&mut self, name: PoolS, typ: TypeName) -> VarID {
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

    pub fn to_member_var(&mut self, id: VarID) {
        self.var_map.get_mut(&id).unwrap().is_member = true
    }

    pub fn new_fun_binding(&mut self, namespace: Namespace, name: PoolS, return_type: TypeName, args: Vec<VarID>) -> Result<FunID, String> {
        let existing = self.find_fun_from_vars(namespace, name, &args);

        if existing.is_some() {
            return Err(format!("function {} already defined with these arguments", name));
        }

        // TODO: Possibly support functions with different arguments. Actually, we probably
        // definitely want to do that...
        let mut output_name = format!("{}{}_args", namespace, name);
        for arg in &args {
            let typ = &self.get_var(*arg).typ;
            output_name.push('_');
            output_name += &typ.to_string();
        }

        let id = FunID(self.grab_id());

        let list = self.reverse_fun_map.entry((namespace, name)).or_insert(vec![]);
        list.push((id, args.clone()));

        self.fun_map.insert(id, FunBinding::new(namespace, output_name, return_type, args));

        Ok(id)
    }

    pub fn find_fun_from_vars(&self, namespace: Namespace, name: PoolS, args: &Vec<VarID>) -> Option<FunID> {
        let options = self.reverse_fun_map.get(&(namespace, name))?;

        for option in options {
            if &option.1 == args {
                return Some(option.0)
            }
        }

        None
    }

    pub fn find_fun_from_compat_nodes<Node : GetExprType>(&self, namespace: Namespace, name: PoolS, args: &Vec<Node>) -> Option<FunID> {
        let options = self.reverse_fun_map.get(&(namespace, name))?;

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

    pub fn find_fun_from_nodes_in_self_namespace<Node : GetExprType>(&self, namespace: Namespace, name: PoolS, args: &Vec<Node>) -> Option<(FunID, bool)> {
        // First: if the function exists in our own namespace, return that function
        let own = self.find_fun_from_compat_nodes(namespace, name, args);
        if let Some(own) = own {
            return Some((own, true));
        }

        // Second: look for the function in super classes. (TODO)

        // Third: look for the function in the global namespace.
        let global = self.find_fun_from_compat_nodes(Namespace::Global, name, args);

        // This is the last option, return it directly.
        return global.map(|global| (global, false));
    }

    pub fn get_fun(&self, id: FunID) -> &FunBinding {
        self.fun_map.get(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    #[allow(unused)]
    pub fn get_fun_mut(&mut self, id: FunID) -> &mut FunBinding {
        self.fun_map.get_mut(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    pub fn fun_bindings(&self) -> Values<'_, FunID, FunBinding> {
        self.fun_map.values()
    }

    pub fn get_type(&self, typename: &TypeName) -> Option<&Type> {
        self.type_map.get(typename)
    }

    pub fn get_or_make_type(&mut self, typename: &TypeName) -> &mut Type {
        if let None = self.type_map.get(typename) {
            self.type_map.insert(typename.clone(), Type::new(typename));
        }
        return self.type_map.get_mut(typename).unwrap();
    }

    pub fn type_push_member(&mut self, typename: &TypeName, member: VarID) {
        let typ = self.get_or_make_type(typename);
        typ.members.push(member);
    }

    pub fn type_bindings(&self) -> Values<'_, TypeName, Type> {
        self.type_map.values()
    }

}