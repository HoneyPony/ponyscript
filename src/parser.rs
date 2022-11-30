use std::io::{Read};
use crate::ast;
use crate::ast::{Func, Node};
use crate::ast::Node::{Empty};

use crate::lexer::{Lexer, Token};
use crate::string_pool::{PoolS};
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

    fn eat_or_err(&mut self, tok: Token, msg: &'static str) -> Result<(), String> {
        if self.eat(tok) {
            Ok(())
        }
        else {
            Err(String::from(msg))
        }
    }

    fn eat_id(&mut self) -> Option<PoolS> {
        if let Token::ID(string) = self.current {
            self.advance();
            return Some(string);
        }
        None
    }

    fn eat_id_or_err(&mut self, msg: &'static str) -> Result<PoolS, String> {
        self.eat_id().ok_or(String::from(msg))
    }

    pub fn new(lexer: Lexer<R>) -> Self {
       Parser {
            lexer,
            current: token::bad()
        }
    }

    fn parse_id_type(&mut self) -> Result<ast::Type, String> {
        let id = self.eat_id_or_err("Expected identifier")?;

        if self.eat(Token::LBracket) {
            let mut inner = vec![];

            loop {
                let next_type = self.parse_type()?;
                inner.push(next_type);

                if !self.eat(Token::Comma) {
                    self.eat_or_err(Token::RBracket, "Expected ',' or ']' in arg list")?;

                    return Ok(ast::Type::Parameterized(id, inner));
                }
            }
        }
        else {
            return Ok(ast::Type::Primitive(id));
        }
    }

    fn parse_type(&mut self) -> Result<ast::Type, String> {
        if self.eat(Token::Plus) {
            return self.parse_id_type().map(|inner| ast::Type::Deref(Box::new(inner)));
        }
        if self.eat(Token::QuestionMark) {
            return self.parse_id_type().map(|inner| ast::Type::Optional(Box::new(inner)));
        }
        return self.parse_id_type();
    }

    fn parse_let(&mut self) -> ast::RNode {
        self.advance();

        let id = self.eat_id_or_err("Expected identifier after let")?;
        self.eat_or_err(Token::Colon, "Expected ':' after identifier")?;

        let typ = self.parse_type()?;
        return Ok(ast::Declaration::new(id, typ).to_node());

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

    fn parse_statement(&mut self) -> ast::RNode {
        match &self.current {
            Token::KeyLet => {
                self.parse_let()
            }
            _ => {
                ast::err("Unknown statement")
            }
        }
    }

    fn parse_fun(&mut self) -> ast::RNode {
        self.advance();

        let id = self.eat_id_or_err("Unexpected token after 'fun'")?;

        let mut result = Func::new(id);

        self.eat_or_err(Token::LParen,"Expected '(' after function name")?;

        while let Token::ID(param) = self.current {
            self.eat_or_err(Token::Colon,"Expected ':' after function parameter name")?;
            let next_type = self.parse_type()?;
            result.args.push((param, next_type));
        }

        self.eat_or_err(Token::RParen, "Expected ')' after function name")?;
        self.eat_or_err(Token::Colon,"Expected ':' after function name")?;
        self.eat_or_err(Token::BlockStart,"Expected block after function")?;

        while !self.eat(Token::BlockEnd) {
            let statement = self.parse_statement();
            result.body.push(statement?);
        }

        return result.to_rnode();
    }

    fn parse_top_level(&mut self) -> ast::RNode {
        match self.current {
            Token::EOF => Ok(Empty),
            Token::KeyFun => self.parse_fun(),
            _ => {
                self.advance();
                ast::err("Unexpected token at top level. Expected 'fun'")
            }
        }
    }

    pub fn parse(&mut self) -> ast::RNode {
        self.advance();

        let mut tree = ast::Tree { children: vec![] };

        while self.current.is_something() {
            tree.children.push(self.parse_top_level()?);
        }

        Ok(Node::Tree(tree))
    }
}