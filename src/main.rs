mod lexer;
mod string_pool;
mod parser;
mod ast;

use std::io::stdout;
use crate::parser::Parser;

fn main() {
    let string =
"fun test():
\tlet x : int
\tlet y : int";

    let mut parser = Parser::from_str(string);
    let tree = parser.parse();

    dbg!(&tree);

    ast::codegen(&tree, &mut stdout()).expect("Failed to write");
}
