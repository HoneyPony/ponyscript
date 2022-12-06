use crate::bindings::Bindings;
use super::*;

pub fn codegen<W: Write>(bindings: &mut Bindings, node: &Node, writer: &mut W) -> io::Result<()> {
    match node {
        Node::Func(f) => {
            writer.write_fmt(format_args!("{} {}() {{\n", f.return_type, f.name))?;
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
            let binding = bindings.get(dec.bind_id);
            writer.write_fmt(format_args!("{} {};\n", binding.output_name, binding.typ))?;
        }
        Node::Assign(bind, expr) => {
            writer.write_fmt(format_args!("{} = ", bind))?;
            codegen(bindings,expr.as_ref(), writer)?;
            writer.write(b";\n")?;
        }
        Node::NumConst(str) => {
            writer.write_fmt(format_args!("{}", str.value_str))?;
        }
        _ => {

        }
    }
    Ok(())
}