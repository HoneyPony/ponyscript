use super::*;

pub fn add(lhs: UntypedNode, rhs: UntypedNode) -> RNode {
    Ok(Node::BinOp(Op::Add, Box::new(lhs), Box::new(rhs)))
}