mod lexer;
mod string_pool;
mod parser;
mod ast;
mod bindings;

use std::fs::File;
use std::io;
use std::io::{BufReader, stdout};
use crate::bindings::Bindings;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::string_pool::StringPool;

fn main() {
  /*  let string =
"fun test() -> float:
\tlet x : int
\tlet y : int
\tlet z : Set[int, +Node]";

    let mut parser = Parser::from_str(string);
    let tree = parser.parse();

    dbg!(&tree);

    if let Ok(tree) = tree {
        ast::codegen(&tree, &mut stdout()).expect("Failed to write");
    }*/
    let file = File::open("examples/test.pony.script").unwrap();
    let mut pool = StringPool::new();
    let mut lexer = Lexer::new(&pool,String::from("examples/test.pony.script"), BufReader::new(file));
    let mut bindings = Bindings::new();
    let mut debug = Parser::new(lexer, &mut bindings);

    let tree = debug.parse();

    dbg!(&tree);

    if let Ok(tree) = tree {
        ast::codegen(&mut bindings,&tree, &mut stdout()).expect("Could not codegen to stdout");
    }
}
