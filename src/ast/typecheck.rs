use std::ops::Deref;
use crate::ast::{BindPoint, Node, Type, TypedNode, UntypedNode};
use crate::ast::Node::Tree;
use crate::bindings::{Bindings, GetID, VarID};

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

fn propagate_numeric(node: &mut TypedNode, typ: &Type) {
    match node {
        Node::NumConst(val, num_typ) => {
            if num_typ == &Type::UnspecificNumeric {
                *num_typ = typ.clone();
            }
        }
        Node::BinOp(_, lhs, rhs) => {
            propagate_numeric(rhs, typ);
            propagate_numeric(lhs, typ);
        }
        Node::FunCall(_, called_on, _, args) => {
            if let Some(called_on) = called_on {
                propagate_numeric(called_on, typ);
            }
            for arg in args {
                propagate_numeric(arg, typ);
            }
        }
        _ => {}
    }
}

fn typecheck_assignment(bindings: &mut Bindings, expr: UntypedNode, id: VarID) -> Result<Box<TypedNode>, String> {
    let mut checked_expr = typecheck(bindings, expr)?;
    let bound = bindings.get_var_mut(id);
    if type_match_var(&mut bound.typ, &checked_expr.1) {
        // Var is matched to type, try propagating type to RHS
        if bound.typ.is_specific_numeric() && checked_expr.1 == Type::UnspecificNumeric {
            propagate_numeric(&mut checked_expr.0, &bound.typ);
        }

        // Just return the type, will be wrapped in typecheck()
        return Ok(Box::new(checked_expr.0));
    }
    return Err(String::from("Could not match types"));
}

pub fn typecheck<'a>(bindings: &mut Bindings, node: UntypedNode) -> Result<(TypedNode, Type), String> {
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

            return Ok((Node::Tree(typed_tree), Type::Error));
        }
        Node::FunDecl(id, body) => {
            let body: Result<Vec<TypedNode>, String> = body.into_iter().map(|node| {
                let next = typecheck(bindings, node)?;
                Ok(next.0)
            }).collect();

            let body = body?;

            return Ok((Node::FunDecl(id, body), Type::Error));
        }
        Node::Decl(id, expr) => {
            let expr = expr.map(|expr| typecheck_assignment(bindings, *expr, id));
            let expr = match expr {
                Some(Err(e)) => return Err(e),
                Some(Ok(expr)) => Some(expr),
                None => None
            };
            let node : TypedNode = Node::Decl(id, expr);
            return Ok((node, Type::Error));
        }
        Node::Assign(bind, expr) => {
            //let _expr_type = typecheck(bindings, expr)?;
            // TODO: Re-update bindings? Is this possible from here??

            match bind {
                BindPoint::Unbound(_) => {
                    return Err(String::from("Unbound ID"));
                }
                BindPoint::BoundTo(id) => {
                    let expr = typecheck_assignment(bindings, *expr, id)?;
                    let node : TypedNode = Node::Assign(id, expr);
                    return Ok((node, Type::Error));
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

            if left.1 == Type::UnspecificNumeric && right.1.is_specific_numeric() {
                propagate_numeric(&mut left.0, &right.1);
                left.1 = right.1.clone();
            }
            else if right.1 == Type::UnspecificNumeric && left.1.is_specific_numeric() {
                propagate_numeric(&mut right.0, &left.1);
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
        Node::FunCall(namespace, called_on, point, args) => {
            let args: Result<_, String> = args.into_iter().map(|arg| {
                let arg = typecheck(bindings, arg)?;
                Ok(arg.0)
            }).collect();

            let args = args?;

            // Typecheck the called on arg as well
            let called_on = match called_on {
                Some(inner) => {
                    let inner = typecheck(bindings, *inner)?.0;
                    Some(Box::new(inner))
                }
                None => None
            };

            // If there is a called on: Then, we need to match based on that.
            match called_on {
                Some(called_on) => {
                    /* TODO */
                    return Err(String::from("Not implemented yet, sorry!"));
                }
                None => {
                    // If there is no called_on, then we need to find using the "self"
                    // parameter (if it is a dynamic call...) and otherwise use global
                    // functions.

                    match point {
                        BindPoint::Unbound(name) => {
                            let binding = bindings
                                .find_fun_from_nodes_in_self_namespace(namespace, name, &args)
                                .ok_or(format!("In call to {}, could not find matching arg list", name))?;

                            // If binding.1, then we need to change called_on to be a SelfRef node.
                            let called_on = if binding.1 {
                                Some(Box::new(TypedNode::SelfRef))
                            }
                            else {
                                None
                            };

                            let typ = bindings.get_fun(binding.0).return_type.clone();
                            return Ok((
                                Node::FunCall(namespace, called_on, binding.0, args),
                                typ
                            ));
                        }
                        BindPoint::BoundTo(id) => {
                            let typ = bindings.get_fun(id).return_type.clone();
                            return Ok((
                                Node::FunCall(namespace, called_on,id, args),
                                typ
                            ));
                        }
                    }
                }
            }
        }
        // TODO: SelfRef needs to know the type of the current self. This needs to be done
        // I guess, using a per-file basis...?
        Node::SelfRef => { return Ok((Node::SelfRef, Type::Error)) }
        Node::Empty => { return Ok((Node::Empty, Type::Error)) }

    }
}