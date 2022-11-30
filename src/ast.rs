use std::fmt::{Debug, DebugList, Formatter};
use std::io;
use std::io::Write;
use crate::string_pool::PoolS;

mod types;
pub use types::Type;

pub struct Tree {
    pub children: Vec<Node>
}

pub struct Func {
    pub name: PoolS,
    pub args: Vec<(PoolS, PoolS)>,
    pub body: Vec<Node>
}

impl Func {
    pub fn new(name: PoolS) -> Self {
        Func {
            name,
            args: vec![],
            body: vec![]
        }
    }

    pub fn to_node(self) -> Node {
        return Node::Func(self)
    }
}

pub struct Declaration {
    pub name: PoolS,
    pub typ: Type
}

impl Declaration {
    pub fn new(name: PoolS, typ: Type) -> Self {
        Declaration {
            name,
            typ
        }
    }

    pub fn to_node(self) -> Node { return Node::Decl(self) }
}

pub enum Node {
    Tree(Tree),
    Func(Func),
    Decl(Declaration),
    ParseError(String),
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
            Node::ParseError(err) => {
                f.write_fmt(format_args!("[err '{}']", err))?;
            }
            _ => { f.write_str("[unknown]")?; }
        }
        Ok(())
    }
}

pub fn err(string: &'static str) -> Node {
    Node::ParseError(String::from(string))
}

pub fn codegen<W: Write>(node: &Node, writer: &mut W) -> io::Result<()> {
    match node {
        Node::Func(f) => {
            writer.write_fmt(format_args!("void {}() {{\n", f.name))?;
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
        _ => {

        }
    }
    Ok(())
}