use crate::string_pool::PoolS;

pub struct Tree {
    pub children: Vec<Node>
}

pub struct Func {
    name: PoolS
}

pub enum Node {
    Tree(Tree),
    Func(Func),
    ParseError(String),
    Empty
}

pub fn err(string: &'static str) -> Node {
    Node::ParseError(String::from(string))
}

pub fn func(name: PoolS) -> Node {
    let f = Func{name};
    Node::Func(f)
}