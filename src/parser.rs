use std::io::{Read};
use crate::ast;
use crate::ast::{Node, TypeName};
use crate::ast::Node::{Empty};
use crate::bindings::{Bindings, FunID, Namespace, VarID};

use crate::lexer::{Lexer, Token};
use crate::string_pool::{PoolS, StringPool};
use crate::lexer::token;
use crate::parser::scope::Scopes;

mod scope;

enum ParserContext {
    Unknown,
    NewClass(TypeName)
}

pub struct Parser<'a, R: Read> {
    lexer: Lexer<'a, R>,

    current: Token,

    bindings: &'a mut Bindings,

    scope: Scopes,

    namespace: Namespace,

    context: ParserContext
}

impl<'a> Parser<'a, &[u8]> {
    #[allow(unused)]
    pub fn from_str(pool: &'a StringPool, string: &'static str, bindings: &'a mut Bindings) -> Self {
        Parser::new(Lexer::from_str(pool,string), bindings)
    }
}

impl<'a, R: Read> Parser<'a, R> {
    fn advance(&mut self) {
        self.current = self.lexer.next();
    }

    fn bind_var(&mut self, string: PoolS) -> ast::BindPoint<VarID> {
        self.scope.find_var(string)
    }

    fn unresolved_fun(&mut self, name: PoolS) -> ast::BindPoint<FunID> {
        ast::BindPoint::<FunID>::unresolved(name)
    }

    fn handle_top_level_var_binding(&mut self, id: VarID) {
        match &self.context {
            ParserContext::Unknown => {
                eprintln!("ponyscript compiler error: trying to parse top-level variable with unknown context");
                std::process::exit(-1);
            }
            ParserContext::NewClass(name) => {
                // IMPORTANT TODO: Run type inference on top-level let statements that don't have
                // anything else happening...
                // Maybe they could be added to the tree, and simply not codegen'd...?
                self.bindings.type_push_member(&name, id);
                self.bindings.to_member_var(id);
            }
        }
    }

    fn new_var_binding(&mut self, string: PoolS, typ: TypeName) -> VarID {
        let id = self.bindings.new_var_binding(string, typ);
        self.scope.add_var(string, id);
        if self.scope.is_top_level() {
            self.handle_top_level_var_binding(id);
        }
        id
    }

    fn eat(&mut self, tok: Token) -> bool {
        if self.current == tok {
            self.advance();
            true
        }
        else { false }
    }

    fn err(&self, msg: &'static str) -> ast::RNode {
        Err(self.lexer.err_msg(msg))
    }

    fn eat_or_err(&mut self, tok: Token, msg: &'static str) -> Result<(), String> {
        if self.eat(tok) {
            Ok(())
        }
        else {
            Err(self.lexer.err_msg(msg))
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
        self.eat_id().ok_or(self.lexer.err_msg(msg))
    }

    pub fn new(lexer: Lexer<'a, R>, bindings: &'a mut Bindings) -> Self {
       Parser {
            lexer,
            current: token::bad(),
            bindings,
            scope: Scopes::new(),
            namespace: Namespace::Global,
            context: ParserContext::Unknown
        }
    }

    fn parse_id_type(&mut self) -> Result<ast::TypeName, String> {
        let id = self.eat_id_or_err("Expected type")?;

        if self.eat(Token::LBracket) {
            let mut inner = vec![];

            loop {
                let next_type = self.parse_type()?;
                inner.push(next_type);

                if !self.eat(Token::Comma) {
                    self.eat_or_err(Token::RBracket, "Expected ',' or ']' in arg list")?;

                    return Ok(ast::TypeName::Parameterized(id, inner));
                }
            }
        }
        else {
            return Ok(ast::TypeName::Primitive(id).to_specific());
        }
    }

    fn parse_type(&mut self) -> Result<ast::TypeName, String> {
        if self.eat(Token::Plus) {
            return self.parse_id_type().map(|inner| ast::TypeName::Deref(Box::new(inner)));
        }
        if self.eat(Token::QuestionMark) {
            return self.parse_id_type().map(|inner| ast::TypeName::Optional(Box::new(inner)));
        }
        return self.parse_id_type();
    }

    fn parse_expr(&mut self) -> ast::RNode {
        let lhs = match self.current {
            Token::Num(str) => {
                self.advance();
                Ok(Node::NumConst(str, ast::TypeName::UnspecificNumeric))
            },
            Token::ID(_) => {
                self.parse_expr_id()
            }
            _ => { self.err("Expected expression") }
        }?;

        match self.current {
            Token::Plus => {
                self.advance();
                let rhs = self.parse_expr()?;

                ast::op::add(lhs, rhs)
            }
            _ => { Ok(lhs) }
        }
    }

    fn parse_let(&mut self) -> ast::RNode {
        self.advance();

        let id = self.eat_id_or_err("Expected identifier after let")?;

        let mut typ = ast::TypeName::Unset;

        if self.eat(Token::Colon) {
            typ = self.parse_type()?;
        }

        if self.eat(Token::Equals) {
            let expr = self.parse_expr()?;
            let expr = Some(Box::new(expr));
            let bind_id = self.new_var_binding(id, typ);
            return Ok(Node::Decl(bind_id, expr));
        }
        else {
            let bind_id = self.new_var_binding(id, typ);
            return Ok(Node::Decl(bind_id, None));
        }
    }

    fn parse_expr_id(&mut self) -> ast::RNode {
        let id = self.eat_id_or_err("Failed to consume identifier when parsing identifier")?;
        if self.eat(Token::LParen) {
            let mut args = vec![];

            if !self.eat(Token::RParen) {
                // Only look for arguments if there isn't an immediate right parenthesis
                loop {
                    args.push(self.parse_expr()?);

                    if !self.eat(Token::Comma) {
                        if self.eat(Token::RParen) {
                            break;
                        }
                        return self.err("Expected ')' or ',' in function call");
                    }
                }
            }
            // We can't actually bind to a specific function call yet, even if we have seen it...
            // In particular, resolving which function to bind to has to be done with type information.
            return Ok(Node::FunCall(self.namespace, None,self.unresolved_fun(id), args));
        }
        else {
            return Ok(Node::VarRef(self.bind_var(id)));
        }
    }

    fn parse_statement_id(&mut self) -> ast::RNode {
        let lhs = self.parse_expr_id()?;

        if self.eat(Token::Equals) {
            let rhs = self.parse_expr()?;

            match lhs {
                Node::VarRef(var) => {
                    return Ok(Node::Assign(var, Box::new(rhs)));
                }
                _ => {
                    return Err(format!("Only variable assignment supported at the moment"));
                }
            }
        }

        // Function calls are valid statements even if there is no equals
        if let Node::FunCall(_, _, _, _) = lhs {
            return Ok(lhs);
        }

        // If there is no assignment and no function call, it's not a valid statement (for now).

        self.err("Expected function call or arithmetic expression")
    }

    fn parse_statement(&mut self) -> ast::RNode {
        match &self.current {
            Token::KeyLet => {
                self.parse_let()
            }
            Token::ID(_) => {
                self.parse_statement_id()
            }
            _ => {
                self.err("Unknown statement")
            }
        }
    }

    fn parse_fun_impl(&mut self) -> ast::RNode {
        self.advance();

        let id = self.eat_id_or_err("Unexpected token after 'fun'")?;

        let mut args = vec![];
        let mut return_type = TypeName::Void;

        self.eat_or_err(Token::LParen,"Expected '(' after function name")?;

        while let Token::ID(param) = self.current {
            self.advance();
            self.eat_or_err(Token::Colon,"Expected ':' after function parameter name")?;
            let next_type = self.parse_type()?;

            let var = self.new_var_binding(param, next_type);
            args.push(var);

            if !self.eat(Token::Comma) {
                break;
            }
        }

        self.eat_or_err(Token::RParen, "Expected ')' after function name")?;

        // Return type comes after arrow, before colon
        if self.eat(Token::RArrow) {
            return_type = self.parse_type()?;
        }

        self.eat_or_err(Token::Colon,"Expected ':' after function")?;
        self.eat_or_err(Token::BlockStart,"Expected block after function")?;

        let func_id = self.bindings.new_fun_binding(self.namespace, id, return_type, args)?;
        let mut body = vec![];

        while !self.eat(Token::BlockEnd) {
            let statement = self.parse_statement();
            body.push(statement?);
        }

        return Ok(Node::FunDecl(func_id, body));
    }

    fn parse_fun(&mut self) -> ast::RNode {
        self.scope.push();
        let result = self.parse_fun_impl();
        self.scope.pop();
        result
    }

    fn parse_top_level(&mut self) -> ast::RNode {
        match self.current {
            Token::EOF => Ok(Empty),
            Token::KeyFun => self.parse_fun(),
            // Top-level variable declarations should not be codegen'd for now (global variables later, maybe)
            Token::KeyLet => { self.parse_let()?; Ok(Empty) }
            _ => {
                self.advance();
                self.err("Unexpected token at top level. Expected 'fun'")
            }
        }
    }

    pub fn parse(&mut self) -> ast::RNode {
        self.advance();

        self.eat_or_err(Token::KeyExtends, "Expected 'extends' at top of file")?;

        let base = self.eat_id_or_err("Expected base type at top of file")?;

        self.eat_or_err(Token::KeyAs, "Expected 'as' at top of file")?;

        let own = self.eat_id_or_err("Expected node type at top of file")?;

        self.namespace = Namespace::DynamicCall(own);

        let mut tree = ast::Tree { base_type: base, own_type: own, children: vec![] };

        self.bindings.get_or_make_type(&TypeName::Primitive(own));

        self.context = ParserContext::NewClass(TypeName::Primitive(own));

        while self.current.is_something() {
            tree.children.push(self.parse_top_level()?);
        }

        Ok(Node::Tree(tree))
    }
}