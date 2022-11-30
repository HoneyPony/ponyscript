mod lexer;
mod string_pool;
mod parser;
mod ast;

use std::io::stdout;
use crate::parser::Parser;

fn main() {
    let string =
"fun test():
\tlet x := 5
\tlet y := 7

\t
";

    let mut parser = Parser::from_str("fun test():");
    let tree = parser.parse();

    ast::codegen(&tree, &mut stdout()).expect("Failed to write");
}
