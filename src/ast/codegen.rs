use crate::bindings::Bindings;
use super::*;

pub fn codegen<W: Write>(bindings: &mut Bindings, node: &Node, writer: &mut W) -> io::Result<()> {
    match node {
        Node::Func(f) => {
            let fun = bindings.get_fun(f.bind_id);
            writer.write_fmt(format_args!("{} {}() {{\n", fun.return_type, fun.output_name))?;
            for s in &f.body {
                codegen(bindings,s, writer)?;
            }
            writer.write(b"}\n")?;
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
        _ => {

        }
    }
    Ok(())
}