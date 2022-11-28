use std::io::{BufReader, Read};
use crate::ast;
use crate::ast::Node;
use crate::ast::Node::{Empty, ParseError};
use crate::lexer::{Lexer, Token};
use crate::string_pool::StringPool;
use crate::lexer::token;

pub struct Parser<R: Read> {
    lexer: Lexer<R>,

    current: Token
}

impl Parser<&[u8]> {
    pub fn from_str(string: &'static str) -> Self {
        Parser::new(Lexer::from_str(string))
    }
}

impl<R: Read> Parser<R> {
    fn advance(&mut self) {
        self.current = self.lexer.next()
    }

    pub fn new(lexer: Lexer<R>) -> Self {
       Parser {
            lexer,
            current: token::bad()
        }
    }

    fn parse_fun(&mut self) -> ast::Node {
        self.advance();
        if let Token::ID(id) = self.current {
            ast::func(id)
        }
        else { ast::err("Unexpected token after 'fun'") }
    }

    fn parse_top_level(&mut self) -> ast::Node {
        match self.current {
            Token::EOF => Empty,
            Token::KeyFun => self.parse_fun(),
            _ => ast::err("Unexpected token")
        }
    }

    pub fn parse(&mut self) -> ast::Tree {
        let mut tree = ast::Tree { children: vec![] };

        while self.current.is_something() {
            tree.children.push(self.parse_top_level());
        }

        tree
    }
}