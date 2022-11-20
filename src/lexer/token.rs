use std::fmt::{Debug, format, Formatter, Write};
use super::*;

#[derive(Debug)]
#[derive(PartialEq)]

pub enum Token {
    ID(PoolS),
    StringLiteral(Vec<u8>),
    BadLex,
    EOF
}

pub struct WrappedToken<'a, R: Read> {
    pub token: Token,
    source: &'a Lexer<'a, R>
}

impl<'a, R: Read> WrappedToken<'a, R> {
    pub fn new(source: &'a Lexer<R>, token: Token) -> Self {
        return WrappedToken { token, source };
    }

    pub fn is_eof(&self) -> bool { self.token == EOF }

    pub fn is_bad(&self) -> bool { self.token == BadLex }

    pub fn is_id_str(&self, string: &'static str) -> bool {
        self.token == ID(self.source.string_pool.pool_str(string))
    }

    pub fn is_lit_str(&self, string: &'static str) -> bool {
        self.token == StringLiteral(string.as_bytes().to_vec())
    }
}

impl<'a, R: Read> Debug for WrappedToken<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.token {
            ID(ps) => {
                let str = self.source.string_pool.unpool_to_utf8(*ps);
                f.write_fmt(format_args!("Wrapped(ID[{}])", str))
            }
            StringLiteral(arr) => {
                f.write_fmt(format_args!("Wrapped(StringLiteral[{}])", String::from_utf8(arr.clone()).unwrap()))
            }
            EOF => {
                f.write_fmt(format_args!("Wrapped(EOF)"))
            }
            BadLex => {
                f.write_fmt(format_args!("Wrapped(BadLex)"))
            }
        }
        //
        // match tok {
        //     ID(ps) => format!("ID [{}]", self.string_pool.unpool_to_utf8(*ps)),
        //     StringLiteral(arr) => format!("StringLiteral [{}]", String::from_utf8(arr.clone()).unwrap()),
        //     EOF => format!("EOF"),
        //     BadLex => format!("BadLex")
        // }
    }
}

pub fn eof<'a, R: Read>(source: &'a Lexer<R>) -> WrappedToken<'a, R> {
    WrappedToken::new(source, EOF)
}

pub fn bad<'a, R: Read>(source: &'a Lexer<R>) -> WrappedToken<'a, R> {
    WrappedToken::new(source, BadLex)
}

pub fn id<'a, R: Read>(source: &'a Lexer<R>, bytes: &Vec<u8>) -> WrappedToken<'a, R> {
    let id = source.string_pool.pool(bytes);
    WrappedToken::new(source, ID(id))
}

pub fn id_str<'a, R: Read>(source: &'a Lexer<R>, string: &'static str) -> WrappedToken<'a, R> {
    let id = source.string_pool.pool_str(string);
    WrappedToken::new(source, ID(id))
}

pub fn lit<'a, R: Read>(source: &'a Lexer<R>, bytes: Vec<u8>) -> WrappedToken<'a, R> {
    WrappedToken::new(source, StringLiteral(bytes))
}

pub fn lit_str<'a, R: Read>(source: &'a Lexer<R>, string: &'static str) -> WrappedToken<'a, R> {
    WrappedToken::new(source, StringLiteral(string.as_bytes().to_vec()))
}