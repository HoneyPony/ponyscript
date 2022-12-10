use super::*;

pub fn add(lhs: Node, rhs: Node) -> RNode {
    Ok(Node::BinOp(Op::Add, Box::new(lhs), Box::new(rhs)))
}