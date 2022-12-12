use std::io;
use std::io::Write;
use crate::bindings::Bindings;

use super::*;

pub fn write_forward_declarations<W: Write>(bindings: &Bindings, writer: &mut W) -> io::Result<()> {
    for fun in bindings.fun_bindings() {
        codegen_fun_decl(bindings, fun, writer)?;
        writer.write(b";\n")?;
    }

    Ok(())
}