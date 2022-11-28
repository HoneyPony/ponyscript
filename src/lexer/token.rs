use std::fmt::{Debug, format, Formatter, Write};
use super::*;

#[derive(PartialEq)]
pub enum Token {
    ID(PoolS),
    StringLiteral(Vec<u8>),
    Num(PoolS),
    KeyVar,
    KeyFun,
    BadLex,
    EOF
}

impl Token {
    pub fn is_eof(&self) -> bool { self == &EOF }

    pub fn is_bad(&self) -> bool { self == &BadLex }

    pub fn is_something(&self) -> bool {
        !self.is_eof() && !self.is_bad()
    }

    pub fn is_id_str(&self, string: &'static str) -> bool {
        if let ID(str) = self {
            str.eq_utf8(string)
        }
        else { false }
    }

    pub fn is_num_str(&self, string: &'static str) -> bool {
        if let Num(str) = self {
            str.eq_utf8(string)
        }
        else { false }
    }

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
            _ => { f.write_fmt(format_args!("[Unknown, need to implement]")) }
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

pub fn id_str(pool: &StringPool, string: &'static str) -> Token {
    let id = pool.pool_str(string);
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
        b'v' => {
            if &bytes[1..] == b"ar" {
                return KeyVar
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

pub fn lit_str(string: &'static str) -> Token {
    StringLiteral(string.as_bytes().to_vec())
}

pub fn num(pool: &StringPool, bytes: Vec<u8>) -> Token {
    let num = pool.pool(bytes);
    Num(num)
}
