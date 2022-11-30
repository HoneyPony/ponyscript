use std::io::{BufReader, Read};
use crate::ast;
use crate::ast::{Func, Node};
use crate::ast::Node::{Empty, ParseError};
use crate::ast::Type::Error;
use crate::lexer::{Lexer, Token};
use crate::string_pool::{PoolS, StringPool};
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

    fn eat_id(&mut self) -> Option<PoolS> {
        if let Token::ID(string) = self.current {
            self.advance();
            return Some(string);
        }
        None
    }

    pub fn new(lexer: Lexer<R>) -> Self {
       Parser {
            lexer,
            current: token::bad()
        }
    }

    fn parse_id_type(&mut self) -> Option<ast::Type> {
        if let Some(id) = self.eat_id() {
            if self.eat(Token::LBracket) {
                let mut inner = vec![];

                loop {
                    let next_type = self.parse_type();
                    if let Some(next_type) = next_type {
                        inner.push(next_type);
                        if !self.eat(Token::Comma) {
                            if self.eat(Token::RBracket) {
                                return Some(ast::Type::Parameterized(id, inner));
                            } else {
                                return None;
                            }
                        }
                    }
                    else {
                        return None;
                    }
                }
            }
            else {
                return Some(ast::Type::Primitive(id));
            }
        }
        None
    }

    fn parse_type(&mut self) -> Option<ast::Type> {
        if self.eat(Token::Plus) {
            return self.parse_id_type().map(|inner| ast::Type::Deref(Box::new(inner)));
        }
        if self.eat(Token::QuestionMark) {
            return self.parse_id_type().map(|inner| ast::Type::Optional(Box::new(inner)));
        }
        return self.parse_id_type();
    }

    fn parse_let(&mut self) -> ast::Node {
        self.advance();
        if let Some(id) = self.eat_id() {
            if !self.eat(Token::Colon) {
                return ast::err("Expected ':' after identifier");
            }

            if let Some(typ) = self.parse_type() {
                return ast::Declaration::new(id, typ).to_node();
            } else {
                return ast::err("Expected type after ':'");
            }
        }
        ast::err("Expected identifier after let")

        // Code sketch for what would be nicer...
        // let id = self.eat_id_or("Expected identifier after let");
        // id.eat_or(Token::Colon, "Expected colon after identifier")
        // let type = id.eat_id();
        // ...maybe somethign like that?
        //
        //
        // Or. maybe, somethig like,...
        // let id = self.eat_id_or("Expected identifier after let").eat_or(Token::Colon, "Expected colon after id");
        // let typ = self.parse_optional_type();
        // let expr = self.then_eat(Token::Equals).and(|| self.parse_expression());
        // return ast::zip((id, typ, expr), |(id, typ, expr)| ast::Declaration::new(id, typ, expr));
    }

    fn parse_statement(&mut self) -> ast::Node {
        match &self.current {
            Token::KeyLet => {
                self.parse_let()
            }
            _ => {
                ast::err("Unknown statement")
            }
        }
    }

    fn parse_fun(&mut self) -> ast::Node {
        self.advance();
        if let Some(id) = self.eat_id() {
            let mut result = Func::new(id);

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

            if !self.eat(Token::BlockStart) {
                return ast::err("Expected block after function");
            }

            while !self.eat(Token::BlockEnd) {
                let statement = self.parse_statement();
                result.body.push(statement);
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
                ast::err("Unexpected token at top level. Expected 'fun'")
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