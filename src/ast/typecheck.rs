use crate::ast::{BindPoint, Node, Type};
use crate::bindings::Bindings;

fn wrap_option(typ: &Type) -> Type {
    match typ {
        Type::Error => Type::Error,
        _ => Type::Optional(Box::new(typ.clone()))
    }
}

pub fn type_match_var(var_type: &mut Type, expr_type: &Type) -> bool {
    match var_type {
        Type::Unset => {
            match expr_type {
                Type::UnspecificNumeric => {
                    *var_type = Type::Float;
                    true
                }
                _ => {
                    *var_type = expr_type.clone();
                    true
                }
            }
        }
        Type::Optional(inner) => {
            if let Type::Optional(expr_inner) = expr_type {
                return type_match_var(inner, expr_inner);
            }
            else {
                return type_match_var(inner, expr_type);
            }
        }
        Type::Deref(inner) => {
            if let Type::Deref(expr_inner) = expr_type {
                return type_match_var(inner, expr_inner);
            }
            else {
                return type_match_var(inner, expr_type);
            }
        }
        _ => {
            if let Type::UnspecificNumeric = expr_type {
                return var_type.is_specific_numeric();
            }
            return var_type == expr_type
        }
    }
}

fn propagate_numeric(node: &mut Node, typ: &Type) {
    match node {
        Node::NumConst(num) => {
            num.typ = typ.clone();
        }
        Node::BinOp(_, lhs, rhs) => {
            propagate_numeric(rhs, typ);
            propagate_numeric(lhs, typ);
        }
        Node::FunCall(_, args) => {
            for arg in args {
                propagate_numeric(arg, typ);
            }
        }
        _ => {}
    }
}

pub fn typecheck<'a>(bindings: &mut Bindings, node: &mut Node) -> Result<Type, String> {
    match node {
        Node::Tree(nodes) => {
            for node in nodes.children.iter_mut() {
                typecheck(bindings, node)?;
            }
            return Ok(Type::Error);
        }
        Node::FunDecl(f) => {
            for node in f.body.iter_mut() {
                typecheck(bindings, node)?;
            }
            return Ok(Type::Error);
        }
        Node::Decl(decl) => {

            match &mut decl.expr {
                Some(node) => {
                    let expr = typecheck(bindings, node.as_mut())?;
                    let bound = bindings.get_var_mut(decl.bind_id);
                    if type_match_var(&mut bound.typ, &expr) {
                        return Ok(Type::Error); // Not an expression
                    }
                    return Err(String::from("Could not match types"));
                }
                None => { }
            }
        }
        Node::Assign(bind, expr) => {
            let expr_type = typecheck(bindings, expr)?;
            // TODO: Re-update bindings

            match bind {
                BindPoint::Unbound(_) => {
                    return Err(String::from("Unbound ID"));
                }
                BindPoint::BoundTo(id) => {
                    let bound = bindings.get_var_mut(*id);
                    if type_match_var(&mut bound.typ, &expr_type) {
                        // Var is matched to type, try propagating type to RHS
                        if bound.typ.is_specific_numeric() && expr_type == Type::UnspecificNumeric {
                            propagate_numeric(expr, &bound.typ);
                        }

                        return Ok(Type::Error); // Not an expression
                    }
                    return Err(String::from("Could not match types in assign"));
                }
            }
        }
        Node::NumConst(num) => {
            return Ok(num.typ.clone());
        }
        Node::BinOp(_, lhs, rhs) => {
            let mut left = typecheck(bindings, lhs)?;
            let mut right = typecheck(bindings, rhs)?;

            if left == Type::UnspecificNumeric && right.is_specific_numeric() {
                propagate_numeric(lhs, &right);
                left = right.clone();
            }
            else if right == Type::UnspecificNumeric && left.is_specific_numeric() {
                propagate_numeric(rhs, &left);
                right = left.clone();
            }

            if left == right {
                return Ok(left);
            }
            return Err(String::from("Could not match types in binary expression"));
        }
        Node::FunCall(_, _) => {}
        Node::Empty => {}
    }
    return Ok(Type::Error);
}