use std::fmt::{Debug, Display, Formatter, Pointer};
use std::io;
use std::io::Write;
use crate::string_pool::PoolS;

mod types;
pub use types::Type;

pub enum Binding {
    Unbound(PoolS),
    BoundTo(i64)
}

impl Display for Binding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::Unbound(s) => {
                f.write_fmt(format_args!("{}", s))
            },
            Binding::BoundTo(i64) => {
                f.write_str("[todo]")
            }
        }
    }
}

pub struct Tree {
    pub children: Vec<Node>
}

pub struct Func {
    pub name: PoolS,
    pub return_type: Type,
    pub args: Vec<(PoolS, Type)>,
    pub body: Vec<Node>
}

impl Func {
    pub fn new(name: PoolS) -> Self {
        Func {
            name,
            return_type: Type::Void,
            args: vec![],
            body: vec![]
        }
    }

    pub fn to_node(self) -> Node {
        return Node::Func(self)
    }

    pub fn to_rnode(self) -> RNode {
        return Ok(self.to_node())
    }
}

pub struct Declaration {
    pub name: PoolS,
    pub typ: Type,
    pub expr: Option<Box<Node>>
}

impl Declaration {
    pub fn new(name: PoolS, typ: Type) -> Self {
        Self::new_expr(name, typ, None)
    }

    pub fn new_expr(name: PoolS, typ: Type, expr: Option<Box<Node>>) -> Self {
        Declaration {
            name,
            typ,
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

pub enum Node {
    Tree(Tree),
    Func(Func),
    Decl(Declaration),
    Assign(Binding, Box<Node>),
    NumConst(NumConst),
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
            Node::Func(func) => {
                f.write_fmt(format_args!("[func '{}']", func.name))?;
            }
            _ => { f.write_str("[unknown]")?; }
        }
        Ok(())
    }
}

pub type RNode = Result<Node, String>;

pub fn codegen<W: Write>(node: &Node, writer: &mut W) -> io::Result<()> {
    match node {
        Node::Func(f) => {
            writer.write_fmt(format_args!("{} {}() {{\n", f.return_type, f.name))?;
            for s in &f.body {
                codegen(s, writer)?;
            }
            writer.write(b"}\n")?;
        }
        Node::Tree(tree) => {
            for child in &tree.children {
                codegen::<W>(child, writer)?;
            }
        }
        Node::Decl(dec) => {
            writer.write_fmt(format_args!("{} {};\n", dec.typ, dec.name))?;
        }
        Node::Assign(bind, expr) => {
            writer.write_fmt(format_args!("{} = ", bind))?;
            codegen(expr.as_ref(), writer)?;
            writer.write(b";\n")?;
        }
        Node::NumConst(str) => {
            writer.write_fmt(format_args!("{}", str.value_str))?;
        }
        _ => {

        }
    }
    Ok(())
}