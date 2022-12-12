use std::fmt::{Debug, Formatter};
use super::*;

#[derive(PartialEq)]
pub enum Token {
    ID(PoolS),
    StringLiteral(Vec<u8>),
    Num(PoolS),
    BlockStart,
    BlockEnd,
    LParen,
    RParen,
    Colon,
    Plus,
    Minus,
    Equals,
    RArrow,
    QuestionMark,
    LBracket,
    RBracket,
    Comma,
    KeyLet,
    KeyFun,
    KeyExtends,
    KeyAs,
    BadLex,
    EOF
}

impl Token {
    pub fn is_eof(&self) -> bool { self == &EOF }

    pub fn is_bad(&self) -> bool { self == &BadLex }

    pub fn is_something(&self) -> bool {
        !self.is_eof() && !self.is_bad()
    }

    #[allow(unused)]
    pub fn is_block_start(&self) -> bool { self == &BlockStart }

    #[allow(unused)]
    pub fn is_block_end(&self) -> bool { self == &BlockEnd }

    #[allow(unused)]
    pub fn is_id_str(&self, string: &'static str) -> bool {
        if let ID(str) = self {
            str.eq_utf8(string)
        }
        else { false }
    }

    #[allow(unused)]
    pub fn is_num_str(&self, string: &'static str) -> bool {
        if let Num(str) = self {
            str.eq_utf8(string)
        }
        else { false }
    }

    #[allow(unused)]
    pub fn is_lit_str(&self, string: &'static str) -> bool {
        self == &StringLiteral(string.as_bytes().to_vec())
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ID(ps) => {
                f.write_fmt(format_args!("[ID '{}']", ps.to_utf8()))
            }
            StringLiteral(arr) => {
                f.write_fmt(format_args!("[StringLiteral '{}']", String::from_utf8(arr.clone()).unwrap()))
            }
            Num(ps) => {
                f.write_fmt(format_args!("[Num '{}']", ps.to_utf8()))
            }
            EOF => {
                f.write_fmt(format_args!("[EOF]"))
            }
            BadLex => {
                f.write_fmt(format_args!("[BadLex]"))
            }

            BlockStart => { f.write_str("[BlockStart]") }
            BlockEnd => { f.write_str("[BlockEnd]") }
            LParen => { f.write_str("[(]") }
            RParen => { f.write_str("[)]") }
            Colon => { f.write_str("[:]") }
            KeyLet => { f.write_str("[KeyLet]") }
            KeyFun => { f.write_str("[KeyFun]") }
            KeyAs => { f.write_str("[KeyAs]") }
            KeyExtends => { f.write_str("[KeyExtends]") }
            Plus => { f.write_str("[+]") }
            QuestionMark => { f.write_str("[?]") }
            LBracket => { f.write_str("[[]") }
            RBracket => { f.write_str("[]]") }
            Equals => { f.write_str("[=]") }
            Comma => { f.write_str("[,]") }
            Minus => { f.write_str("[-]") }
            RArrow => { f.write_str("[->]") }
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

pub fn eof() -> Token {
    EOF
}

pub fn bad() -> Token{
    BadLex
}

pub fn id(pool: &StringPool, bytes: Vec<u8>) -> Token {
    let id = pool.pool(bytes);
    ID(id)
}

pub fn id_or_key(pool: &StringPool, bytes: Vec<u8>) -> Token {
    if bytes.is_empty() {
        return id(pool, bytes);
    }

    match bytes[0] {
        b'f' => {
            if &bytes[1..] == b"un" {
                return KeyFun
            }
            id(pool, bytes)
        }
        b'l' => {
            if &bytes[1..] == b"et" {
                return KeyLet
            }
            id(pool, bytes)
        }
        b'e' => {
            if &bytes[1..] == b"xtends" {
                return KeyExtends
            }
            id(pool, bytes)
        }
        b'a' => {
            if &bytes[1..] == b"s" {
                return KeyAs
            }
            id(pool, bytes)
        }
        _ => {
            id(pool, bytes)
        }
    }
}

pub fn lit(bytes: Vec<u8>) -> Token {
    StringLiteral(bytes)
}

pub fn num(pool: &StringPool, bytes: Vec<u8>) -> Token {
    let num = pool.pool(bytes);
    Num(num)
}
