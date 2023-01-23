use crate::bindings::{Bindings, FunBinding};
use super::*;

mod prelude;
mod forward;

pub use prelude::write_prelude;
pub use forward::write_forward_declarations;

/// Writes all parts of the function declaration, including the return type, parameter types, and
/// parameter names, as well as the closing parenthesis. Does not write a brace or a semicolon,
/// however.
fn codegen_fun_decl<W: Write>(bindings: &Bindings, fun: &FunBinding, writer: &mut W) -> io::Result<()> {
    writer.write_fmt(format_args!("{} {}(", fun.return_type, fun.output_name))?;

    let mut generate_comma = false;

    match fun.namespace {
        Namespace::DynamicCall(typ) => {
            writer.write(b"void *self_ptr")?;
            generate_comma = true;
        }
        _ => { }
    }

    for param in &fun.args {
        if generate_comma {
            writer.write(b", ")?;
        }

        let binding = bindings.get_var(*param);
        writer.write_fmt(format_args!("{} {}", binding.typ, binding.output_name))?;

        generate_comma = true;
    }
    writer.write(b")")?;

    Ok(())
}

pub fn codegen<W: Write>(bindings: &Bindings, node: &TypedNode, writer: &mut W) -> io::Result<()> {
    match node {
        Node::FunDecl(bind_id, body) => {
            let fun = bindings.get_fun(*bind_id);
            codegen_fun_decl(bindings, fun, writer)?;

            writer.write(b" {\n")?;

            match fun.namespace {
                Namespace::DynamicCall(node_name) => {
                    let type_name = TypeName::Primitive(node_name);
                    writer.write_fmt(format_args!("{0} *self = ({0}*)(self_ptr);\n", type_name))?;
                }
                _ => {}
            }

            for statement in body {
                codegen(bindings,statement, writer)?;
            }
            writer.write(b"}\n")?;
        }
        Node::FunCall(namespace, called_on, fun, args) => {
            let fun = bindings.get_fun(*fun);

            writer.write_fmt(format_args!("{}(", fun.output_name))?;
            let mut generate_comma = false;

            // called_on is the first argument.
            if let Some(called_on) = called_on {
                codegen(bindings, called_on, writer)?;
                generate_comma = true;
            }

            for arg in args {
                if generate_comma { writer.write(b", ")?; }

                codegen(bindings, arg, writer)?;

                generate_comma = true;
            }
            writer.write(b");\n")?;
        }
        Node::Tree(tree) => {
            for child in &tree.children {
                codegen::<W>(bindings,child, writer)?;
            }
        }
        Node::Decl(bind_id, expr) => {
            let binding = bindings.get_var(*bind_id);
            writer.write_fmt(format_args!("{} {}", binding.typ, binding.output_name))?;

            if let Some(expr) = expr {
                writer.write(b" = ")?;
                codegen(bindings, expr, writer)?;
            }

            writer.write(b";\n")?;
        }
        Node::Assign(bind, expr) => {
            let binding = bindings.get_var(*bind);
            writer.write_fmt(format_args!("{} = ", binding.output_name))?;
            codegen(bindings, expr.as_ref(), writer)?;
            writer.write(b";\n")?;
        }
        Node::NumConst(val, _) => {
            writer.write_fmt(format_args!("{}", val))?;
        }
        Node::BinOp(op, lhs, rhs) => {
            codegen_op(bindings, op, lhs, rhs, writer)?;
        }
        Node::VarRef(id) => {
            let binding = bindings.get_var(*id);
            writer.write_fmt(format_args!("{}", binding.output_name))?;
        }
        Node::SelfRef => {
            writer.write(b"self")?;
        }
        _ => {

        }
    }
    Ok(())
}

fn codegen_op<W: Write>(bindings: &Bindings, op: &Op, lhs: &Box<TypedNode>, rhs: &Box<TypedNode>, writer: &mut W) -> io::Result<()> {
    // Write the operator function name. This could even allow user-defined operators...
    writer.write_fmt(format_args!("{}_op_{}(", lhs.get_expr_type(bindings), op.impl_str()))?;

    // Write the operator arguments
    codegen(bindings, lhs, writer)?;
    writer.write(b", ")?;
    codegen(bindings, rhs, writer)?;

    writer.write(b")")?;

    Ok(())
}