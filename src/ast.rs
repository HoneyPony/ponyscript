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
use crate::bindings::{Bindings, FunID, Namespace, VarID};

pub enum BindPoint<Id> {
    Unbound(PoolS),
    BoundTo(Id)
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

pub struct Tree {
    pub base_type: PoolS,
    pub own_type: PoolS,
    pub children: Vec<Node>
}

pub struct FunDecl {
    pub bind_id: FunID,
    pub body: Vec<Node>
}

impl FunDecl {
    pub fn new(bind_id: FunID) -> Self {
        FunDecl {
            bind_id,
            body: vec![]
        }
    }

    pub fn to_node(self) -> Node {
        return Node::FunDecl(self)
    }

    pub fn to_rnode(self) -> RNode {
        return Ok(self.to_node())
    }
}

pub struct Declaration {
    pub bind_id: VarID,
    pub expr: Option<Box<Node>>
}

impl Declaration {
    pub fn new(bind_id: VarID) -> Self {
        Self::new_expr(bind_id, None)
    }

    pub fn new_expr(bind_id: VarID, expr: Option<Box<Node>>) -> Self {
        Declaration {
            bind_id,
            expr
        }
    }

    pub fn to_node(self) -> Node { return Node::Decl(self) }

    #[allow(unused)]
    pub fn to_rnode(self) -> RNode {
        return Ok(self.to_node())
    }
}

pub struct NumConst {
    pub value_str: PoolS,
    pub typ: Type
}

impl NumConst {
    pub fn new(value_str: PoolS, typ: Type) -> Self {
        NumConst { value_str, typ }
    }

    pub fn to_node(self) -> Node {
        Node::NumConst(self)
    }
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

pub enum Node {
    Tree(Tree),
    FunDecl(FunDecl),
    Decl(Declaration),
    Assign(BindPoint<VarID>, Box<Node>),
    VarRef(BindPoint<VarID>),
    NumConst(NumConst),
    FunCall(Namespace, BindPoint<FunID>, Vec<Node>),
    BinOp(Op, Box<Node>, Box<Node>),
    Empty
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Tree(tree) => {
                f.write_str("[tree ")?;
                for node in &tree.children {
                    node.fmt(f)?;
                }
                f.write_str("]")?;
            }
            Node::FunDecl(_) => {
                f.write_fmt(format_args!("[func]"))?;
            }
            _ => { f.write_str("[unknown]")?; }
        }
        Ok(())
    }
}

impl Node {
    pub fn get_expr_type(&self, bindings: &Bindings) -> Type {
        match &self {
            Node::Tree(_) => { Type::Error }
            Node::FunDecl(_) => {
                Type::Error
            }
            Node::Decl(_) => { Type::Error }
            Node::Assign(_, _) => { Type::Error }
            Node::NumConst(num) => {
                num.typ.clone()
            }
            Node::VarRef(point) => {
                match point {
                    BindPoint::Unbound(_) => Type::Error,
                    BindPoint::BoundTo(bind_id) => bindings.get_var(*bind_id).typ.clone()
                }
            }
            Node::FunCall(_, point, _) => {
                match point {
                    BindPoint::Unbound(_) => Type::Error,
                    BindPoint::BoundTo(bind_id) => bindings.get_fun(*bind_id).return_type.clone()
                }
            }
            Node::BinOp(_, lhs, _) => {
                lhs.get_expr_type(bindings)
            }
            Node::Empty => { Type::Error }
        }
    }
}

pub type RNode = Result<Node, String>;