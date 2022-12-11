use std::io;
use std::io::Write;


pub fn write_prelude<W: Write>(writer: &mut W) -> io::Result<()> {
    let prelude =
br##"#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>

#define int32_t_op_add(a, b) ((a) + (b))
#define float_op_add(a, b) ((a) + (b))
"##;

    writer.write(prelude)?;

    Ok(())
}