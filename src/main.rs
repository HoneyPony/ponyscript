mod lexer;
mod string_pool;
mod parser;
mod ast;
mod bindings;
mod compiler;

use std::path::{Path, PathBuf};
use crate::compiler::Compiler;

fn main() {
    let mut compiler = Compiler::new();

    let source_paths = vec!["examples/test.pony.script"];

    let parse_errors: Vec<String> = source_paths.iter().map(|path| {
        compiler.parse_source_file(PathBuf::from(path))
    }).filter(|result| result.is_err()).map(|error| error.unwrap_err()).collect();

    if !parse_errors.is_empty() {
        for error in parse_errors {
            println!("{}", error);
        }
        return;
    }

    if let Err(type_err) = compiler.typecheck() {
        println!("Typecheck error: {}", type_err);
        return;
    }

    compiler.output().expect("Failed to output code!");
}
