use std::any::Any;
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::zip;
use crate::ast::{BindPoint, GetExprType, Node, TypeName, TypeBindPoint};
use crate::string_pool::{PoolS, StringPool};

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

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct TypeID(u64);

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

impl GetID<Self> for TypeID {
    fn get_id(&self) -> Option<Self> {
        Some(*self)
    }
}

#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
#[allow(unused)]
pub enum Namespace {
    Global,
    StaticCall(TypeID),
    DynamicCall(TypeID)
}
/*
impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Namespace::Global => f.write_str("g_"),
            Namespace::StaticCall(typ) => f.write_fmt(format_args!("s{}_", typ)),
            Namespace::DynamicCall(typ) => f.write_fmt(format_args!("d{}_", typ))
        }
    }
}*/

pub struct VarBinding<TyBind> {
    pub output_name: String,
    pub typ: TyBind
}

impl VarBinding<TypeBindPoint> {
    pub fn new(output_name: String, typ: TypeName) -> Self {
        VarBinding { output_name, typ: BindPoint::unresolved(typ) }
    }
}

pub struct FunBinding<TyBind> {
    pub output_name: String,
    pub return_type: TyBind,
    pub args: Vec<VarID>,
    pub called_on: Option<TypeID>
}

impl FunBinding<TyBind> {
    pub fn new(output_name: String, return_type: TypeName, args: Vec<VarID>) -> Self {
        FunBinding { output_name, return_type, args, called_on: None }
    }
}

enum Type {
    Node(PoolS),
    Option(TypeID),

    Int32,
    Float,
    Void,
    Error,
    UnspecificNumeric
}

pub struct TypeBinding {
    pub associated_type: TypeName,
    pub output_name: String,

    pub data: Type
}

pub struct Bindings<TyBind> {
    next: u64,
    var_map: HashMap<VarID, VarBinding<TyBind>>,
    fun_map: HashMap<FunID, FunBinding<TyBind>>,
    reverse_fun_map: HashMap<(Namespace, PoolS), Vec<(FunID, Vec<VarID>)>>,
    type_map: HashMap<TypeID, TypeBinding>,
    type_id_map: HashMap<TypeName, TypeID>,
    names: HashMap<PoolS, u64>,

    type_int: TypeID,
    type_float: TypeID,
    type_unspecific_num: TypeID,
    type_void: TypeID,
    type_error: TypeID
}

pub type UntypedBindings = Bindings<TypeBindPoint>;
pub type TypedBindings = Bindings<TypeID>;

impl<TyBind> Bindings<TyBind> {
    pub fn new(pool: &mut StringPool) -> Self {
        let mut bindings = Bindings {
            next: 0,
            var_map: HashMap::new(),
            fun_map: HashMap::new(),
            reverse_fun_map: HashMap::new(),
            type_map: HashMap::new(),
            type_id_map: HashMap::new(),
            names: HashMap::new(),

            type_int: TypeID(0),
            type_float: TypeID(0),
            type_unspecific_num: TypeID(0),
            type_void: TypeID(0),
            type_error: TypeID(0)
        };

        bindings.type_int = bindings.new_type(
            TypeName::Primitive(pool.pool_str("int")),
            Type::Int32);

        bindings.type_float = bindings.new_type(
            TypeName::Primitive(pool.pool_str("float")),
            Type::Float);

        bindings.type_unspecific_num = bindings.new_type(
            TypeName::UnspecificNumeric,
            Type::UnspecificNumeric);

        bindings.type_void = bindings.new_type(
            TypeName::Primitive(pool.pool_str("void")),
            Type::Void);

        bindings.type_error = bindings.new_type(
            TypeName::Error,
            Type::Error);

        bindings
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

    pub fn get_var(&self, id: VarID) -> &VarBinding<TyBind> {
        self.var_map.get(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    pub fn get_var_mut(&mut self, id: VarID) -> &mut VarBinding<TyBind> {
        self.var_map.get_mut(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    fn initialize_type(&mut self, id: TypeID, associated: TypeName, data: Type) {
        let output_name = format!("{}", associated);

        let binding = TypeBinding {
            associated_type: associated,
            output_name,
            data
        };

        self.type_map.insert(id, binding);
    }

    fn new_type(&mut self, typ: TypeName, data: Type) -> TypeID {
        let new = TypeID(self.grab_id());
        self.type_id_map.insert(typ.clone(), new);
        self.initialize_type(new, typ, data);

        new
    }

    pub fn new_node_type(&mut self, name: PoolS) -> TypeID {
        let new = TypeID(self.grab_id());
        self.type_id_map.insert(TypeName::Primitive(name), new);
        self.initialize_type(new, TypeName::Primitive(name), Type::Node(name));

        new
    }

    pub fn find_type_id(&mut self, typ: &TypeName) -> Option<TypeID> {
        let existing = self.type_id_map.get(typ).map(|id| *id);

        if let Some(_) = existing {
            return existing;
        }

        match typ {
            TypeName::Optional(inner) => {
                return self.find_type_id(inner).map(|id|{
                    self.new_type(typ.clone(), Type::Option(id))
                })
            }
            TypeName::Deref(_) => {}
            TypeName::Parameterized(_, _) => {}
            TypeName::Void => {}
            TypeName::UnspecificNumeric => {}
            _ => {}
        }

        None
    }

    fn format_namespace(&self, namespace: &Namespace) -> String {
        match namespace {
            Namespace::Global => String::from("g_"),
            Namespace::DynamicCall(id) => format!("d{}_", self.get_type(*id).output_name),
            Namespace::StaticCall(id) => format!("s{}_", self.get_type(*id).output_name)
        }
    }

    pub fn new_fun_binding(&mut self, namespace: Namespace, name: PoolS, return_type: TypeName, args: Vec<VarID>) -> Result<FunID, String> {
        let existing = self.find_fun_from_vars(namespace, name, &args);

        if existing.is_some() {
            return Err(format!("function {} already defined with these arguments", name));
        }

        // TODO: Possibly support functions with different arguments. Actually, we probably
        // definitely want to do that...
        let mut output_name = format!("{}{}_args", self.format_namespace(&namespace), name);
        for arg in &args {
            let typ = &self.get_var(*arg).typ;
            output_name.push('_');
            output_name += &typ.to_string();
        }

        let id = FunID(self.grab_id());

        let list = self.reverse_fun_map.entry((namespace, name)).or_insert(vec![]);
        list.push((id, args.clone()));

        self.fun_map.insert(id, FunBinding::new(output_name, return_type, args));

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

    pub fn get_type(&self, id: TypeID) -> &TypeBinding {
        self.type_map.get(&id).unwrap()
    }

    pub fn get_fun(&self, id: FunID) -> &FunBinding<TyBind> {
        self.fun_map.get(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    #[allow(unused)]
    pub fn get_fun_mut(&mut self, id: FunID) -> &mut FunBinding<TyBind> {
        self.fun_map.get_mut(&id).unwrap() // TODO: Determine if this unwrap is safe
    }

    pub fn fun_bindings(&self) -> Values<'_, FunID, FunBinding<TyBind>> {
        self.fun_map.values()
    }

    pub fn int(&self) -> TypeID { self.type_int }
    pub fn float(&self) -> TypeID { self.type_float }
    pub fn unspecific_numeric(&self) -> TypeID { self.type_unspecific_num }
    pub fn void(&self) -> TypeID { self.type_void }

    pub fn is_unspecific_numeric(&self, type_id: TypeID) -> bool {
        type_id == self.unspecific_numeric()
    }

    pub fn is_specific_numeric(&self, type_id: TypeID) -> bool {
        let typ = self.get_type(type_id);
        match typ.data {
            Type::Int32 => { true }
            Type::Float => { true }
            _ => { false }
        }
    }

    fn resolve_type_binding(&mut self, point: &TypeBindPoint) -> Option<TypeID> {
        match point {
            BindPoint::Unbound(name) => {
                self.find_type_id(name)
            }
            BindPoint::BoundTo(id) => Some(*id)
        }
    }
}

impl Bindings<TypeBindPoint> {
    pub fn resolve_types(mut self) -> Result<Bindings<TypeID>, String> {
        let mut result = Bindings {
            next: 0,
            var_map: HashMap::new(),
            fun_map: HashMap::new(),
            reverse_fun_map: self.reverse_fun_map,
            type_map: self.type_map,
            type_id_map: self.type_id_map,
            names: self.names,

            type_int: self.type_int,
            type_float: self.type_float,
            type_unspecific_num: self.type_unspecific_num,
            type_void: self.type_void
        };

        let var_map: Result<_, String> = self.var_map.into_iter().map(|(key, value)| {
            let id = result.resolve_type_binding(&value.typ);
            let id = id.ok_or(format!("Could not find type in variable"))?;

            Ok((key, id))
        }).collect();

        result.var_map = var_map?;

        let fun_map: Result<_, String> = self.fun_map.into_iter().map(|(key, value)| {
            let id = result.resolve_type_binding(&value.return_type);
            let id = id.ok_or(format!("Could not find type in function return"))?;

            Ok((key, id))
        }).collect();

        result.fun_map = fun_map?;

        Ok(result)
    }
}