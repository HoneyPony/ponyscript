use std::io::{BufReader, Read};

use crate::string_pool::{PoolS, StringPool};

pub mod token;
mod matcher;
mod predicates;

pub use token::Token;

use token::Token::*;
use predicates::*;
use matcher::Matcher;

pub struct Lexer<R: Read> {
    string_pool: StringPool,
    reader: BufReader<R>,

    /// The current character that the Lexer has read in from the stream. Should be checked against
    /// until some part of the logic wants to advance the stream further.
    current: Option<u8>,

    block_level: i32
}

// fn is(byte: u8) -> fn(Option<u8>) -> bool {
//     |c: Option<u8>| -> bool {
//         match c {
//             Some(other) => byte == other,
//             None => false
//         }
//     }
// }

impl Lexer<&[u8]> {
    pub fn from_str(string: &'static str) -> Self {
        let reader = BufReader::new(string.as_bytes());
        Lexer { reader, string_pool: StringPool::new(), current: Some(b' '), block_level: 0 }
    }
}

impl<R: Read> Matcher for Lexer<R> {
    fn peek(&self) -> Option<u8> {
        self.current
    }

    fn advance(&mut self) -> Option<u8> {
        let mut byte = [0];

        let result = self.current;

        self.current = self.reader.read(&mut byte).ok().map(|read| {
            if read == 1 { Some(byte[0]) } else { None }
        }).flatten();

        result
    }
}

impl<R: Read> Lexer<R> {
    pub fn new(reader: BufReader<R>) -> Self {
        Lexer { reader, string_pool: StringPool::new(), current: Some(b' '), block_level: 0 }
    }

    fn try_match_whitespace(&mut self) -> Option<i32> {
        let mut block_level = 0;
        while self.match_one(b'\t') {
            block_level += 1;
        }
        while self.match_fn(is_whitespace_but_newline).is_some() {}
        if self.match_one(b'\n') {
            while self.match_one(b'\n') {}
            return None;
        }
        Some(block_level)
    }

    fn match_whitespace(&mut self) -> i32 {
        loop {
            let next = self.try_match_whitespace();
            if let Some(block_level) = next {
                return block_level;
            }
        }
    }

    pub fn next(&mut self) -> Token {
        let cur_block_level = self.match_whitespace();
        if cur_block_level > self.block_level {
            self.block_level += 1;
            return Token::BlockStart;
        }
        if cur_block_level < self.block_level {
            self.block_level -= 1;
            return Token::BlockEnd;
        }

        if self.peek().is_none() {
            return token::eof();
        }

        if let Some(mut id) = self.match_to_vec(is_alpha) {
            self.match_onto_vec(&mut id, is_alphanum);

            return token::id_or_key(&self.string_pool, id);
        }

        if let Some(mut num) = self.match_to_vec(is_num) {
            self.match_onto_vec(&mut num, is_num);

            return token::num(&self.string_pool,num);
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
                return token::bad();
            }

            return token::lit( result);
        }

        if self.match_one(b'(') {
            return Token::LParen;
        }
        if self.match_one(b')') {
            return Token::RParen;
        }
        if self.match_one(b':') {
            return Token::Colon;
        }

        token::bad()
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token::BadLex;
    use crate::string_pool::StringPool;
    use super::{Lexer, Token};

    #[test]
    fn lex_id() {
        let mut lexer = Lexer::from_str("  abc    hello    AlphaBET canhave12345 mix12and09");

        assert!(lexer.next().is_id_str("abc"));
        assert!(lexer.next().is_id_str("hello"));
        assert!(lexer.next().is_id_str("AlphaBET"));
        assert!(lexer.next().is_id_str("canhave12345"));
        assert!(lexer.next().is_id_str("mix12and09"));
    }

    #[test]
    fn lex_ascii_string_literal() {
        let mut lexer = Lexer::from_str("    \"string literal\"   \"12__34__5\"    \"!@#$cvbn*()_=|\"   ");

        assert!(lexer.next().is_lit_str("string literal"));
        assert!(lexer.next().is_lit_str("12__34__5"));
        assert!(lexer.next().is_lit_str("!@#$cvbn*()_=|"));
    }

    #[test]
    fn lex_lit_backspace() {
        let mut lexer = Lexer::from_str("   \"\\n\\b\\c\\d\\\"asdf\\\"asdf\"");

        assert!(lexer.next().is_lit_str("\\n\\b\\c\\d\\\"asdf\\\"asdf"));
    }

    #[test]
    fn lex_lit_no_end() {
        let mut lexer = Lexer::from_str("    \"oops, no quote");

        assert!(lexer.next().is_bad());
    }

    #[test]
    fn lex_num() {
        let mut lexer = Lexer::from_str("10   30  20   1531897");

        assert!(lexer.next().is_num_str("10"));
        assert!(lexer.next().is_num_str("30"));
        assert!(lexer.next().is_num_str("20"));
        assert!(lexer.next().is_num_str("1531897"));
    }
}