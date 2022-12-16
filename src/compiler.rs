use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{PathBuf};
use std::process::{Command, Stdio};
use crate::ast::{codegen, Node, Type, typecheck, TypedNode, UntypedNode};
use crate::bindings::Bindings;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::string_pool::StringPool;

pub enum Output {
    Stdout,
    TccLib(PathBuf)
}

impl Output {
    fn codegen(&self, compiler: &Compiler<TypedNode>) -> std::io::Result<()> {
        match &self {
            Output::Stdout => {
                let out = &mut std::io::stdout();
                compiler.codegen_impl(out)?;
            }
            Output::TccLib(lib_file) => {

                let mut child = Command::new("tcc")
                    .stdin(Stdio::piped())
                    .arg("-shared")
                    .arg("-o")
                    .arg(lib_file)
                    .arg("-")
                    .spawn()?;

                let mut stdin = child.stdin.take().expect("Failed to attach to tcc");
                compiler.codegen_impl(&mut stdin)?;
                drop(stdin);

                let ecode = child.wait()?;
                println!("error code = {}", ecode);
            }
        }

        Ok(())
    }
}

pub struct Compiler<'a, Node> {
    pool: &'a StringPool,
    bindings: Bindings,
    trees: Vec<Node>,

    output: Output
}

impl<'a> Compiler<'a, UntypedNode> {
    pub fn new(pool: &'a StringPool, output: Output) -> Self {
        Compiler {
            pool,
            bindings: Bindings::new(),
            trees: vec![],

            output
        }
    }

    pub fn parse_source_file(&mut self, path: &PathBuf) -> Result<(), String> {
        let file = File::open(&path).map_err(|error| error.to_string())?;

        let lexer = Lexer::new(&self.pool,
                                   path.to_string_lossy().to_string(),
                                   BufReader::new(file));
        let mut parser = Parser::new(lexer, &mut self.bindings);

        let tree = parser.parse()?;

        self.trees.push(tree);

        Ok(())
    }

    pub fn typecheck(mut self) -> Result<Compiler<'a, TypedNode>, String> {
        let trees: Result<_, String> = self.trees.into_iter().map(|tree| {
            let tree = typecheck(&mut self.bindings, tree)?;
            Ok(tree.0)
        }).collect();

        let trees = trees?;

        Ok(Compiler::<TypedNode> {
            pool: self.pool,
            bindings: self.bindings,
            output: self.output,
            trees
        })
    }


}

impl<'a> Compiler<'a, TypedNode> {
    pub fn output(&self) -> std::io::Result<()> {
        self.output.codegen(&self)
    }

    /// Actually implements the codegen itself, with a generic Writer type. This function is called
    /// by Output with the correct writer.
    fn codegen_impl<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        codegen::write_prelude(writer)?;
        codegen::write_forward_declarations(&self.bindings, writer)?;

        for tree in self.trees.iter() {
             codegen::codegen(&self.bindings, tree, writer)?;
        }

        Ok(())
    }
}