use std::io;
use std::io::Write;
use crate::bindings::Bindings;

use super::*;

pub fn write_forward_declarations<W: Write>(bindings: &Bindings, writer: &mut W) -> io::Result<()> {
    for fun in bindings.fun_bindings() {
        codegen_fun_decl(bindings, fun, writer)?;
        writer.write(b";\n")?;
    }

    for typ in bindings.type_bindings() {
        writer.write_fmt(format_args!("#define FieldList_{}", typ.output_name))?;

        if let Some(base_class) = &typ.base_class {
            writer.write_fmt(format_args!(" \\\nFieldList_{}", base_class))?;
        }

        for var in &typ.members {
            let var = bindings.get_var(*var);
            writer.write_fmt(format_args!(" \\\n{} {};", var.typ.to_raw_name(), var.output_name))?;
        }

        writer.write(b"\n")?;
    }

    for typ in bindings.type_bindings() {
        writer.write_fmt(format_args!("typedef struct {} {{\n", typ.associated_typename.to_struct_name()))?;
        writer.write_fmt(format_args!("\tFieldList_{}\n", typ.output_name))?;
        writer.write_fmt(format_args!("}} {};\n", typ.output_name))?;
    }

    Ok(())
}