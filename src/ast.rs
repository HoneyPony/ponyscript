use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::Write;
use crate::string_pool::PoolS;

mod types;
pub mod codegen;
mod typecheck;
pub mod op;
pub use types::Type;
pub use codegen::codegen;
pub use typecheck::typecheck;
use crate::bindings::{Bindings, FunID, GetID, Namespace, VarID};

pub enum BindPoint<Id> {
    Unbound(PoolS),
    BoundTo(Id)
}

impl<Id : Copy> GetID<Id> for BindPoint<Id> {
    fn get_id(&self) -> Option<Id> {
        match &self {
            BindPoint::Unbound(_) => None,
            BindPoint::BoundTo(id) => Some(*id)
        }
    }
}

impl<Id> Display for BindPoint<Id> {
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

impl<Id> BindPoint<Id> {
    pub fn unresolved(name: PoolS) -> Self {
        BindPoint::Unbound(name)
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
    fn get_expr_type(&self, bindings: &Bindings) -> Type;
}

pub enum Node<VarBind : GetID<VarID>, FunBind : GetID<FunID>> {
    Tree(Tree<Self>),
    FunDecl(FunID, Vec<Self>),
    Decl(VarID, Option<Box<Self>>),
    Assign(VarBind, Box<Self>),
    VarRef(VarBind),
    NumConst(PoolS, Type),
    FunCall(Namespace, FunBind, Vec<Self>),
    BinOp(Op, Box<Self>, Box<Self>),
    Empty
}

impl<V : GetID<VarID>, F : GetID<FunID>> GetExprType for Node<V, F> {
    fn get_expr_type(&self, bindings: &Bindings) -> Type {
        match &self {
            Node::NumConst(_, typ) => {
                typ.clone()
            }
            Node::VarRef(point) => {
                point.get_id().map_or(Type::Error, |id| bindings.get_var(id).typ.clone())
            }
            Node::FunCall(_, point, _) => {
                point.get_id().map_or(Type::Error, |id| bindings.get_fun(id).return_type.clone())
            }
            Node::BinOp(_, lhs, _) => {
                lhs.get_expr_type(bindings)
            }
            _ => Type::Error
        }
    }
}

pub type UntypedNode = Node<
    BindPoint<VarID>,
    BindPoint<FunID>,
>;

pub type TypedNode = Node<
    VarID,
    FunID,
>;

pub type RNode = Result<UntypedNode, String>;