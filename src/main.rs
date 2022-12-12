mod lexer;
mod string_pool;
mod parser;
mod ast;
mod bindings;
mod compiler;

use std::collections::VecDeque;
use std::env::args;
use std::path::{PathBuf};
use crate::compiler::{Compiler, Output};

struct Config {
    output: Output,
    source_paths: Vec<PathBuf>
}

fn compile(config: Config) {
    let mut compiler = Compiler::new(config.output);

    let parse_errors: Vec<String> = config.source_paths.iter().map(|path| {
        compiler.parse_source_file(path)
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

fn main() {
    let mut config = Config {
        output: Output::Stdout,
        source_paths: vec![]
    };

    let mut args: VecDeque<String> = args().collect();

    // Ignore the program name
    args.pop_front();

    if let Some(arg) = args.front() {
        if arg == "-tcclib" {
            args.pop_front();
            if let Some(output_name) = args.pop_front() {
                config.output = Output::TccLib(PathBuf::from(output_name));
            }
        }
    }

    for arg in args {
        config.source_paths.push(PathBuf::from(arg));
    }

    compile(config);
}
