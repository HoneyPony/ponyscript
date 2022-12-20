use std::ops::Deref;
use crate::ast::{BindPoint, Node, TypeName, TypedNode, UntypedNode};
use crate::ast::Node::Tree;
use crate::bindings::{Bindings, GetID, TypedBindings, TypeID, VarID};

pub fn type_match_var(var_type: &mut TypeName, expr_type: &TypeName) -> bool {
    match var_type {
        TypeName::Unset => {
            match expr_type {
                TypeName::UnspecificNumeric => {
                    *var_type = TypeName::Float;
                    true
                }
                _ => {
                    *var_type = expr_type.clone();
                    true
                }
            }
        }
        TypeName::Optional(inner) => {
            if let TypeName::Optional(expr_inner) = expr_type {
                return type_match_var(inner, expr_inner);
            }
            else {
                return type_match_var(inner, expr_type);
            }
        }
        TypeName::Deref(inner) => {
            if let TypeName::Deref(expr_inner) = expr_type {
                return type_match_var(inner, expr_inner);
            }
            else {
                return type_match_var(inner, expr_type);
            }
        }
        _ => {
            if let TypeName::UnspecificNumeric = expr_type {
                return var_type.is_specific_numeric();
            }
            return var_type == expr_type
        }
    }
}

fn propagate_numeric(node: &mut TypedNode, typ: TypeID) {
    match node {
        Node::NumConst(val, num_typ) => {
            if num_typ == &TypeName::UnspecificNumeric {
                *num_typ = typ;
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

fn typecheck_assignment(bindings: &mut TypedBindings, expr: UntypedNode, id: VarID) -> Result<(TypedNode, TypeID), String> {
    let mut checked_expr = typecheck(bindings, expr)?;
    let bound = bindings.get_var_mut(id);
    if type_match_var(&mut bound.typ, &checked_expr.1) {
        // Var is matched to type, try propagating type to RHS
        if bound.typ.is_specific_numeric() && checked_expr.1 == TypeName::UnspecificNumeric {
            propagate_numeric(&mut checked_expr.0, &bound.typ);
        }

        // It isn't an expression, return Type::Error + typed node
        return Ok((
            Node::Decl(id, Some(Box::new(checked_expr.0))),
            TypeName::Error
        ));
    }
    return Err(String::from("Could not match types"));
}

pub fn typecheck<'a>(bindings: &mut TypedBindings, node: UntypedNode) -> Result<(TypedNode, TypeID), String> {
    match node {
        Node::Tree(tree) => {
            let mut typed_tree = crate::ast::Tree::<TypedNode> {
                base_type: tree.base_type,
                own_type: tree.own_type,
                children: vec![]
            };

            let children: Result<Vec<TypedNode>, String> = tree.children.into_iter().map(|node| {
                let next = typecheck(bindings, node)?;
                Ok(next.0)
            }).collect();

            typed_tree.children = children?;

            return Ok((Node::Tree(typed_tree), TypeName::Error));
        }
        Node::FunDecl(id, body) => {
            let body: Result<Vec<TypedNode>, String> = body.into_iter().map(|node| {
                let next = typecheck(bindings, node)?;
                Ok(next.0)
            }).collect();

            let body = body?;

            return Ok((Node::FunDecl(id, body), TypeName::Error));
        }
        Node::Decl(id, expr) => {

            match expr {
                Some(expr) => {
                    return typecheck_assignment(bindings, *expr, id);
                }
                None => {
                    return Ok((Node::Decl(id, None), TypeName::Error))
                }
            }
        }
        Node::Assign(bind, expr) => {
            //let _expr_type = typecheck(bindings, expr)?;
            // TODO: Re-update bindings? Is this possible from here??

            match bind {
                BindPoint::Unbound(_) => {
                    return Err(String::from("Unbound ID"));
                }
                BindPoint::BoundTo(id) => {
                    return typecheck_assignment(bindings, *expr, id);
                }
            }
        }
        Node::NumConst(id, typ) => {
            return Ok((Node::NumConst(id, typ.clone()), typ));
        }
        Node::VarRef(point) => {
            match point {
                BindPoint::Unbound(_) => {
                    return Err(String::from("Unbound ID"));
                }
                BindPoint::BoundTo(id) => {
                    let typ = bindings.get_var(id).typ.clone();

                    return Ok((
                        Node::VarRef(id),
                        typ
                    ));
                }
            }
        }
        Node::BinOp(op, lhs, rhs) => {
            let mut left = typecheck(bindings, *lhs)?;
            let mut right = typecheck(bindings, *rhs)?;

            if bindings.is_unspecific_numeric(left.1) && bindings.is_specific_numeric(right.1) {
                propagate_numeric(&mut left.0, right.1);
                left.1 = right.1.clone();
            }
            else if bindings.is_unspecific_numeric(right.1) && bindings.is_specific_numeric(left.1) {
                propagate_numeric(&mut right.0, left.1);
                right.1 = left.1.clone();
            }

            if left.1 == right.1 {
                return Ok((
                    Node::BinOp(op, Box::new(left.0), Box::new(right.0)),
                    left.1
                ));
            }
            return Err(String::from("Could not match types in binary expression"));
        }
        Node::FunCall(namespace, point, args) => {
            let args: Result<_, String> = args.into_iter().map(|arg| {
                let arg = typecheck(bindings, arg)?;
                Ok(arg.0)
            }).collect();

            let args = args?;

            match point {
                BindPoint::Unbound(name) => {
                    let binding = bindings
                        .find_fun_from_compat_nodes(namespace,name, &args)
                        .ok_or(format!("In call to {}, could not find matching arg list", name))?;

                    let typ = bindings.get_fun(binding).return_type.clone();
                    return Ok((
                        Node::FunCall(namespace, binding, args),
                        typ
                    ));
                }
                BindPoint::BoundTo(id) => {
                    let typ = bindings.get_fun(id).return_type.clone();
                    return Ok((
                        Node::FunCall(namespace, id, args),
                        typ
                    ));
                }
            }
        }
        Node::TyBindUnused(id) => { return Ok((Node::TyBindUnused(id.get_id().unwrap()), TypeName::Error))}
        Node::Empty => { return Ok((Node::Empty, TypeName::Error)) }
    }
}