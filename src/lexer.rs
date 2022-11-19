use std::io::{BufReader, Read};

use crate::string_pool::{PoolS, StringPool};

pub struct Lexer<'a, R: Read> {
    string_pool: &'a StringPool,
    reader: BufReader<R>,

    /// The current character that the Lexer has read in from the stream. Should be checked against
    /// until some part of the logic wants to advance the stream further.
    current: Option<u8>
}

#[derive(Debug)]
#[derive(PartialEq)]

pub enum Token {
    ID(PoolS),
    StringLiteral(Vec<u8>),
    BadLex,
    EOF
}

use Token::*;

fn is_whitespace(byte: Option<u8>) -> bool {
    let byte = byte.unwrap_or(b'0');
    return byte == b' ' || byte == b'\n' || byte == b'\t' || byte == b'\r';
}

fn is_alpha(byte: Option<u8>) -> bool {
    let byte = byte.unwrap_or(b'/');

    let lower = byte >= b'a' && byte <= b'z';
    let upper = byte >= b'A' && byte <= b'Z';

    return lower || upper;
}

fn is_alphanum(byte: Option<u8>) -> bool {
    let byte = byte.unwrap_or(b'/');

    let lower = byte >= b'a' && byte <= b'z';
    let upper = byte >= b'A' && byte <= b'Z';
    let num   = byte >= b'0' && byte <= b'9';

    return lower || upper || num;
}

// fn is(byte: u8) -> fn(Option<u8>) -> bool {
//     |c: Option<u8>| -> bool {
//         match c {
//             Some(other) => byte == other,
//             None => false
//         }
//     }
// }

impl<'a> Lexer<'a, &[u8]> {
    pub fn from_str(string: &'static str, sp: &'a StringPool) -> Self {
        let reader = BufReader::new(string.as_bytes());
        Lexer { reader, string_pool: sp, current: Some(b' ') }
    }
}

impl<'a, R: Read> Lexer<'a, R> {
    fn new(reader: BufReader<R>, sp: &'a StringPool) -> Self {
        Lexer { reader, string_pool: sp, current: Some(b' ') }
    }

    fn advance(&mut self) -> Option<u8> {
        let mut byte = [0];

        let result = self.current;

        self.current = self.reader.read(&mut byte).ok().map(|read| {
            if read == 1 { Some(byte[0]) } else { None }
        }).flatten();

        result
    }

    fn peek(&self) -> Option<u8> {
        self.current
    }

    /// Tries to match the next byte in the input stream to the given function.
    /// If the byte matches, returns Some, otherwise returns None. Or, if the
    /// stream ends, may also return None.
    fn matchf<F>(&mut self, f: F) -> Option<u8>
    where
        F: Fn(Option<u8>) -> bool
    {
        let matches = f(self.current);
        if matches {
            self.advance()
        }
        else {
            None
        }
    }

    fn match_to_vec<F>(&mut self, f: F) -> Option<Vec<u8>>
    where
        F: Fn(Option<u8>) -> bool
    {
        self.matchf(f).map(|byte| vec![byte])
    }

    fn match_onto_vec<F>(&mut self, vector: &mut Vec<u8>, f: F)
    where
        F: Fn(Option<u8>) -> bool
    {
        while let Some(byte) = self.matchf(&f) {
            vector.push(byte);
        }
    }

    pub fn make_id(&self, string: &Vec<u8>) -> Token {
        ID(self.string_pool.pool(string))
    }

    pub fn make_id_str(&self, string: &'static str) -> Token {
        ID(self.string_pool.pool_str(string))
    }

    pub fn make_lit(&self, string: Vec<u8>) -> Token {
        StringLiteral(string)
    }

    pub fn make_lit_str(&self, string: &'static str) -> Token {
        StringLiteral(string.as_bytes().to_vec())
    }

    fn match_one(&mut self, character: u8) -> bool {
        if self.current.map(|c| c == character).unwrap_or(false) {
            self.advance();
            return true;
        }
        false
    }

    fn match_not(&mut self, character: u8) -> Option<u8> {
        // Assume that EOF also does not match. We basically never want to match EOF.
        if self.current.map(|c| c != character).unwrap_or(false) {
            return self.advance()
        }
        None
    }

    pub fn next(&mut self) -> Token {
        while self.matchf(is_whitespace).is_some() {}

        if self.peek().is_none() {
            return EOF;
        }

        if let Some(mut id) = self.match_to_vec(is_alpha) {
            self.match_onto_vec(&mut id, is_alphanum);

            return self.make_id(&id);
        }

        if self.match_one(b'"') {
            let mut result = Vec::<u8>::new();
            while let Some(next) = self.match_not(b'"') {
                result.push(next);
                if next == b'\\' {
                    // If there is a character after a backslash, include it unconditionally...
                    self.advance().map(|c| result.push(c));
                }
            }

            if !self.match_one(b'"') {
                return BadLex;
            }

            return StringLiteral(result);
        }

        BadLex
    }

    pub fn token_to_string(&self, tok: &Token) -> String {
        match tok {
            ID(ps) => format!("ID [{}]", self.string_pool.unpool_to_utf8(*ps)),
            StringLiteral(arr) => format!("StringLiteral [{}]", String::from_utf8(arr.clone()).unwrap()),
            EOF => format!("EOF"),
            BadLex => format!("BadLex")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token::BadLex;
    use crate::string_pool::StringPool;
    use super::{Lexer, Token};

    #[test]
    fn lex_id() {
        let sp = StringPool::new();
        let mut lexer = Lexer::from_str("  abc    hello    AlphaBET canhave12345 mix12and09", &sp);
        assert_eq!(lexer.next(), lexer.make_id_str("abc"));
        assert_eq!(lexer.next(), lexer.make_id_str("hello"));
        assert_eq!(lexer.next(), lexer.make_id_str("AlphaBET"));
        assert_eq!(lexer.next(), lexer.make_id_str("canhave12345"));
        assert_eq!(lexer.next(), lexer.make_id_str("mix12and09"));
    }

    #[test]
    fn lex_ascii_string_literal() {
        let sp = StringPool::new();
        let mut lexer = Lexer::from_str("    \"string literal\"   \"12__34__5\"    \"!@#$cvbn*()_=|\"   ", &sp);

        assert_eq!(lexer.next(), lexer.make_lit_str("string literal"));
        assert_eq!(lexer.next(), lexer.make_lit_str("12__34__5"));
        assert_eq!(lexer.next(), lexer.make_lit_str("!@#$cvbn*()_=|"));
    }

    #[test]
    fn lex_lit_backspace() {
        let sp = StringPool::new();
        let mut lexer = Lexer::from_str("   \"\\n\\b\\c\\d\\\"asdf\\\"asdf\"", &sp);

        assert_eq!(lexer.next(), lexer.make_lit_str("\\n\\b\\c\\d\\\"asdf\\\"asdf"));
    }

    #[test]
    fn lex_lit_no_end() {
        let sp = StringPool::new();
        let mut lexer = Lexer::from_str("    \"oops, no quote", &sp);

        assert_eq!(lexer.next(), BadLex);
    }
}