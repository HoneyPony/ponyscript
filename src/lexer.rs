use std::io::{BufReader, Read};

use crate::string_pool::{PoolS, StringPool};

pub mod token;
mod matcher;
mod predicates;

pub use token::Token;

use token::Token::*;
use predicates::*;
use matcher::Matcher;

pub struct Lexer<'a, R: Read> {
    string_pool: &'a StringPool,
    reader: BufReader<R>,

    /// The current character that the Lexer has read in from the stream. Should be checked against
    /// until some part of the logic wants to advance the stream further.
    current: Option<u8>,

    current_tagline: String, // Used for generating error messages

    current_line: i32, // Used for generating error messages
    current_column: i32,

    block_level: i32,

    matched_block_level: i32,

    // The problem with trying to match whitespace block levels is that it requires a bit of state.
    // In particular, after we match, e.g, some tabs, the lexer is in a state where the next character
    // is not a tab... the exact same situation as when the lexer starts reading a line where there are
    // no tabs at the beginning at all.
    //
    // So, what we must do instead... keep track of when we've seen a newline, as that is the only state
    // when we are allowed to match a new BlockStart or BlockEnd.
    may_match_blocks: bool
}

impl<'a> Lexer<'a, &[u8]> {
    #[allow(unused)]
    pub fn from_str(pool: &'a StringPool, string: &'static str) -> Self {
        let reader = BufReader::new(string.as_bytes());
        Lexer::new(pool,String::from("[string]"), reader)
    }
}

impl<'a, R: Read> Matcher for Lexer<'a, R> {
    fn peek(&self) -> Option<u8> {
        self.current
    }

    fn advance(&mut self) -> Option<u8> {
        let mut byte = [0];

        let result = self.current;

        self.current = self.reader.read(&mut byte).ok().map(|read| {
            if read == 1 { Some(byte[0]) } else { None }
        }).flatten();

        result.map(|byte| {
            if byte == b'\n' {
                self.current_line += 1;
                self.current_column = 1;
            }
            else {
                self.current_column += 1;
            }
        });

        result
    }
}

impl<'a, R: Read> Lexer<'a, R> {
    pub fn new(pool: &'a StringPool, tagline: String, reader: BufReader<R>) -> Self {
        Lexer {
            reader,
            string_pool: pool,
            current: Some(b' '),
            current_tagline: tagline,
            current_line: 1,
            current_column: 1,
            block_level: 0,
            matched_block_level: 0,
            may_match_blocks: true
        }
    }

    pub fn err_msg(&self, message: &'static str) -> String {
        format!("{}:{}:{}: {}", self.current_tagline,
            self.current_line, self.current_column, message)
    }

    fn try_match_whitespace(&mut self) -> Option<i32> {
        let mut block_level = 0;
        while self.match_one(b'\t') {
            block_level += 1;
        }

        while self.match_fn(is_whitespace_but_newline).is_some() {}

        // Skip comments
        if self.match_one(b'#') {
            while self.peek().map(|c| c != b'\n').unwrap_or(false) {
                self.advance();
            }
        }

        if self.match_one(b'\n') {
            // After seeing a newline is the only time the lexer may match blocks.
            self.may_match_blocks = true;
            while self.match_one(b'\n') || self.match_one(b'\r') {}
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

    fn make_block_token(&mut self) -> Option<Token> {
        if self.matched_block_level > self.block_level {
            self.block_level += 1;
            return Some(Token::BlockStart)
        } else if self.matched_block_level < self.block_level {
            self.block_level -= 1;
            return Some(Token::BlockEnd)
        }
        None
    }

    pub fn next(&mut self) -> Token {
        if let Some(block) = self.make_block_token() {
            return block;
        }

        let new_block_level = self.match_whitespace();
        let may_match_blocks = self.may_match_blocks;
        self.may_match_blocks = false; // TODO: Don't do this weird variable juggle
        if may_match_blocks {
            self.matched_block_level = new_block_level;
            if let Some(block) = self.make_block_token() {
                return block;
            }
        }

        if self.peek().is_none() {
            if self.block_level > 0 {
                self.block_level -= 1;
                self.matched_block_level -= 1;
                return Token::BlockEnd;
            }
            return token::eof();
        }

        if let Some(mut id) = self.match_to_vec(is_alpha) {
            self.match_onto_vec(&mut id, is_id_char);

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
        if self.match_one(b'[') {
            return Token::LBracket;
        }
        if self.match_one(b']') {
            return Token::RBracket;
        }
        if self.match_one(b':') {
            return Token::Colon;
        }
        if self.match_one(b'+') {
            return Token::Plus;
        }
        if self.match_one(b'?') {
            return Token::QuestionMark;
        }
        if self.match_one(b',') {
            return Token::Comma;
        }
        if self.match_one(b'=') {
            return Token::Equals;
        }
        if self.match_one(b'-') {
            if self.match_one(b'>') {
                return Token::RArrow;
            }
            return Token::Minus;
        }

        token::bad()
    }
}

#[cfg(test)]
mod tests {
    use crate::string_pool::StringPool;
    use super::{Lexer};
    use super::Token;

    #[test]
    fn lex_id() {
        let mut sp = StringPool::new();
        let mut lexer = Lexer::from_str(&sp,"  abc    hello    AlphaBET canhave12345 mix12and09");

        assert!(lexer.next().is_id_str("abc"));
        assert!(lexer.next().is_id_str("hello"));
        assert!(lexer.next().is_id_str("AlphaBET"));
        assert!(lexer.next().is_id_str("canhave12345"));
        assert!(lexer.next().is_id_str("mix12and09"));
    }

    #[test]
    fn lex_ascii_string_literal() {
        let mut sp = StringPool::new();
        let mut lexer = Lexer::from_str(&sp,"    \"string literal\"   \"12__34__5\"    \"!@#$cvbn*()_=|\"   ");

        assert!(lexer.next().is_lit_str("string literal"));
        assert!(lexer.next().is_lit_str("12__34__5"));
        assert!(lexer.next().is_lit_str("!@#$cvbn*()_=|"));
    }

    #[test]
    fn lex_lit_backspace() {
        let mut sp = StringPool::new();
        let mut lexer = Lexer::from_str(&sp,"   \"\\n\\b\\c\\d\\\"asdf\\\"asdf\"");

        assert!(lexer.next().is_lit_str("\\n\\b\\c\\d\\\"asdf\\\"asdf"));
    }

    #[test]
    fn lex_lit_no_end() {
        let mut sp = StringPool::new();
        let mut lexer = Lexer::from_str(&sp,"    \"oops, no quote");

        assert!(lexer.next().is_bad());
    }

    #[test]
    fn lex_num() {
        let mut sp = StringPool::new();
        let mut lexer = Lexer::from_str(&sp,"10   30  20   1531897");

        assert!(lexer.next().is_num_str("10"));
        assert!(lexer.next().is_num_str("30"));
        assert!(lexer.next().is_num_str("20"));
        assert!(lexer.next().is_num_str("1531897"));
    }

    #[test]
    fn lex_blocks() {
        let mut sp = StringPool::new();
        let mut lexer = Lexer::from_str(&sp,"abc1\n\tabc2\n\t\tabc3");

        assert!(lexer.next().is_id_str("abc1"));
        assert!(lexer.next().is_block_start());
        let next = lexer.next();
        assert!(next.is_id_str("abc2"), "expected abc2 got {:?}", next);
        assert!(lexer.next().is_block_start());
        assert!(lexer.next().is_id_str("abc3"));
        assert!(lexer.next().is_block_end());
        let next = lexer.next();
        assert!(next.is_block_end(), "expected [BlockEnd] got {:?}", next);
    }

    #[test]
    fn lex_plus_minus_arrow() {
        let mut sp = StringPool::new();
        let mut lexer = Lexer::from_str(&sp,"+ - ->");
        assert_eq!(lexer.next(), Token::Plus);
        assert_eq!(lexer.next(), Token::Minus);
        assert_eq!(lexer.next(), Token::RArrow);
    }
}