use crate::bindings::Bindings;
use super::*;

mod prelude;

pub use prelude::write_prelude;

pub fn codegen<W: Write>(bindings: &mut Bindings, node: &Node, writer: &mut W) -> io::Result<()> {
    match node {
        Node::FunDecl(f) => {
            let fun = bindings.get_fun(f.bind_id);
            writer.write_fmt(format_args!("{} {}(", fun.return_type, fun.output_name))?;

            let mut generate_comma = false;
            for param in &fun.args {
                if generate_comma {
                    writer.write(b", ")?;
                }

                let binding = bindings.get_var(*param);
                writer.write_fmt(format_args!("{} {}", binding.typ, binding.output_name))?;

                generate_comma = true;
            }

            writer.write(b") {\n")?;

            for s in &f.body {
                codegen(bindings,s, writer)?;
            }
            writer.write(b"}\n")?;
        }
        Node::FunCall(bind, args) => {
            if let BindPoint::BoundTo(fun) = bind {
                let fun = bindings.get_fun(*fun);

                writer.write_fmt(format_args!("{}(", fun.output_name))?;
                let mut generate_comma = false;
                for arg in args {
                    if generate_comma { writer.write(b", ")?; }

                    codegen(bindings, arg, writer)?;

                    generate_comma = true;
                }
                writer.write(b");\n")?;
            }
        }
        Node::Tree(tree) => {
            for child in &tree.children {
                codegen::<W>(bindings,child, writer)?;
            }
        }
        Node::Decl(dec) => {
            let binding = bindings.get_var(dec.bind_id);
            writer.write_fmt(format_args!("{} {};\n", binding.typ, binding.output_name))?;
        }
        Node::Assign(bind, expr) => {
            if let BindPoint::BoundTo(bind_id) = bind {
                let binding = bindings.get_var(*bind_id);
                writer.write_fmt(format_args!("{} = ", binding.output_name))?;
                codegen(bindings, expr.as_ref(), writer)?;
                writer.write(b";\n")?;
            }
            // TODO: Return an error, maybe...?
        }
        Node::NumConst(str) => {
            writer.write_fmt(format_args!("{}", str.value_str))?;
        }
        Node::BinOp(op, lhs, rhs) => {
            codegen_op(bindings, op, lhs, rhs, writer)?;
        }
        _ => {

        }
    }
    Ok(())
}

fn codegen_op<W: Write>(bindings: &mut Bindings, op: &Op, lhs: &Box<Node>, rhs: &Box<Node>, writer: &mut W) -> io::Result<()> {
    // Write the operator function name. This could even allow user-defined operators...
    writer.write_fmt(format_args!("{}_op_{}(", lhs.get_expr_type(bindings), op.impl_str()))?;

    // Write the operator arguments
    codegen(bindings, lhs, writer)?;
    writer.write(b", ")?;
    codegen(bindings, rhs, writer)?;

    writer.write(b")")?;

    Ok(())
}