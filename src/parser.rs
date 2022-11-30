use std::io::{BufReader, Read};
use crate::ast;
use crate::ast::{Func, Node};
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
        self.current = self.lexer.next();
    }

    fn eat(&mut self, tok: Token) -> bool {
        if self.current == tok {
            self.advance();
            true
        }
        else { false }
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
            let mut result = Func::new(id);
            self.advance();

            if !self.eat(Token::LParen) {
                return ast::err("Expected '(' after function name");
            }

            while let Token::ID(param) = self.current {
                if !self.eat(Token::Colon) {
                    return ast::err("Expected ':' after function parameter name");
                }
                // TODO: Implement actual type parser
                if let Token::ID(typ) = self.current {
                    result.args.push((param, typ))
                }
                else {
                    return ast::err("Expected type name");
                }
            }

            if !self.eat(Token::RParen) {
                return ast::err("Expected ')' after function name");
            }

            if !self.eat(Token::Colon) {
                return ast::err("Expected ':' after function name");
            }



            return result.to_node();
        }

        ast::err("Unexpected token after 'fun'")
    }

    fn parse_top_level(&mut self) -> ast::Node {
        match self.current {
            Token::EOF => Empty,
            Token::KeyFun => self.parse_fun(),
            _ => {
                self.advance();
                ast::err("Unexpected token")
            }
        }
    }

    pub fn parse(&mut self) -> ast::Node {
        self.advance();

        let mut tree = ast::Tree { children: vec![] };

        while self.current.is_something() {
            tree.children.push(self.parse_top_level());
        }

        Node::Tree(tree)
    }
}