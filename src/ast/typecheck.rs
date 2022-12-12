use crate::ast::{BindPoint, Node, Type};
use crate::bindings::{Bindings, VarID};

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
            if num.typ == Type::UnspecificNumeric {
                num.typ = typ.clone();
            }
        }
        Node::BinOp(_, lhs, rhs) => {
            propagate_numeric(rhs, typ);
            propagate_numeric(lhs, typ);
        }
        Node::FunCall(_, _, args) => {
            for arg in args {
                propagate_numeric(arg, typ);
            }
        }
        _ => {}
    }
}

fn typecheck_assignment(bindings: &mut Bindings, expr: &mut Node, id: VarID) -> Result<Type, String> {
    let expr_type = typecheck(bindings, expr)?;
    let bound = bindings.get_var_mut(id);
    if type_match_var(&mut bound.typ, &expr_type) {
        // Var is matched to type, try propagating type to RHS
        if bound.typ.is_specific_numeric() && expr_type == Type::UnspecificNumeric {
            propagate_numeric(expr, &bound.typ);
        }

        return Ok(Type::Error); // Not an expression
    }
    return Err(String::from("Could not match types"));
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
                Some(expr) => {
                    return typecheck_assignment(bindings, expr, decl.bind_id);
                }
                None => { }
            }
        }
        Node::Assign(bind, expr) => {
            let _expr_type = typecheck(bindings, expr)?;
            // TODO: Re-update bindings

            match bind {
                BindPoint::Unbound(_) => {
                    return Err(String::from("Unbound ID"));
                }
                BindPoint::BoundTo(id) => {
                    return typecheck_assignment(bindings, expr, *id);
                }
            }
        }
        Node::NumConst(num) => {
            return Ok(num.typ.clone());
        }
        Node::VarRef(point) => {
            match point {
                BindPoint::Unbound(_) => {
                    return Err(String::from("Unbound ID"));
                }
                BindPoint::BoundTo(id) => {
                    return Ok(bindings.get_var(*id).typ.clone());
                }
            }
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
        Node::FunCall(namespace, point, args) => {
            for arg in args.iter_mut() {
                typecheck(bindings, arg)?;
            }
            match point {
                BindPoint::Unbound(name) => {
                    let binding = bindings
                        .find_fun_from_compat_nodes(*namespace,*name, args)
                        .ok_or(format!("In call to {}, could not find matching arg list", name))?;

                    point.bind_to(binding);

                    return Ok(bindings.get_fun(binding).return_type.clone());
                }
                BindPoint::BoundTo(id) => {
                    return Ok(bindings.get_fun(*id).return_type.clone());
                }
            }
        }
        Node::Empty => {}
    }
    return Ok(Type::Error);
}