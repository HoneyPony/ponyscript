use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::Write;
use crate::string_pool::PoolS;

mod types;
pub mod codegen;
mod typecheck;
pub mod op;
pub use types::TypeName;
pub use codegen::codegen;
pub use typecheck::typecheck;
use crate::bindings::{Bindings, FunID, GetID, Namespace, TypeID, VarID};

pub enum BindPoint<Unres, Id> {
    Unbound(Unres),
    BoundTo(Id)
}

impl<Unres, Id : Copy> GetID<Id> for BindPoint<Unres, Id> {
    fn get_id(&self) -> Option<Id> {
        match &self {
            BindPoint::Unbound(_) => None,
            BindPoint::BoundTo(id) => Some(*id)
        }
    }
}

impl<Unres, Id> Display for BindPoint<Unres, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BindPoint::Unbound(s) => {
                f.write_fmt(format_args!("{}", s))
            },
            BindPoint::BoundTo(_) => {
                f.write_str("[todo]")
            }
        }
    }
}

impl<Unres, Id> BindPoint<Unres, Id> {
    pub fn unresolved(unresolved: Unres) -> Self {
        BindPoint::Unbound(unresolved)
    }

    pub fn bind_to(&mut self, new_binding: Id) {
        let old = std::mem::replace(self, BindPoint::BoundTo(new_binding));
        drop(old);
    }
}

pub struct Tree<Node> {
    pub base_type: PoolS,
    pub own_type: PoolS,
    pub children: Vec<Node>
}

#[allow(unused)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide
}

impl Op {
    pub fn impl_str(&self) -> &'static str {
        match self {
            Op::Add => "add",
            Op::Subtract => "sub",
            Op::Multiply => "mul",
            Op::Divide => "div"
        }
    }
}

pub trait GetExprType {
    fn get_expr_type(&self, bindings: &Bindings) -> TypeName;
}

pub enum Node<VarBind : GetID<VarID>, FunBind : GetID<FunID>, TyBind : GetID<TypeID>> {
    Tree(Tree<Self>),
    FunDecl(FunID, Vec<Self>),
    Decl(VarID, Option<Box<Self>>),
    Assign(VarBind, Box<Self>),
    VarRef(VarBind),
    NumConst(PoolS, TypeID),
    FunCall(Namespace, FunBind, Vec<Self>),
    BinOp(Op, Box<Self>, Box<Self>),
    TyBindUnused(TyBind),
    Empty
}

impl<V : GetID<VarID>, F : GetID<FunID>, T : GetID<TypeID>> GetExprType for Node<V, F, T> {
    fn get_expr_type(&self, bindings: &Bindings) -> TypeID {
        match &self {
            Node::NumConst(_, typ) => {
                *typ
            }
            Node::VarRef(point) => {
                point.get_id().map_or(TypeName::Error, |id| bindings.get_var(id).typ.clone())
            }
            Node::FunCall(_, point, _) => {
                point.get_id().map_or(TypeName::Error, |id| bindings.get_fun(id).return_type.clone())
            }
            Node::BinOp(_, lhs, _) => {
                lhs.get_expr_type(bindings)
            }
            _ => TypeName::Error
        }
    }
}

pub type VarBindPoint = BindPoint<PoolS, VarID>;
pub type FunBindPoint = BindPoint<PoolS, FunID>;
pub type TypeBindPoint = BindPoint<TypeName, TypeID>;

pub type UntypedNode = Node<
    BindPoint<PoolS, VarID>,
    BindPoint<PoolS, FunID>,
    TypeBindPoint
>;

pub type TypedNode = Node<
    VarID,
    FunID,
    TypeID
>;

pub type RNode = Result<UntypedNode, String>;