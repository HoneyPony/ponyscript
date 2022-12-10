use std::fmt::{Debug, Display, Formatter, Pointer};
use std::io;
use std::io::Write;
use crate::string_pool::PoolS;

mod types;
mod codegen;
mod typecheck;
pub mod op;
pub use types::Type;
pub use codegen::codegen;
pub use typecheck::typecheck;
use crate::bindings::{Bindings, FunID, VarID};

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
            BindPoint::BoundTo(i64) => {
                f.write_str("[todo]")
            }
        }
    }
}

pub struct Tree {
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
    NumConst(NumConst),
    FunCall(BindPoint<FunID>, Vec<Node>),
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
            Node::FunDecl(func) => {
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
            Node::FunDecl(fun) => {
                Type::Error
            }
            Node::Decl(_) => { Type::Error }
            Node::Assign(_, _) => { Type::Error }
            Node::NumConst(num) => {
                num.typ.clone()
            }
            Node::FunCall(bind, _) => {
                match bind {
                    BindPoint::Unbound(str) => Type::Error,
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