use std::io::{BufReader, Cursor, Read, Seek};

use crate::string_pool::{PoolS, StringPool};

pub struct Lexer<'a, R: Read + Seek> {
    string_pool: &'a StringPool,
    reader: BufReader<R>
}

#[derive(Debug)]
#[derive(PartialEq)]

pub enum Token {
    Empty,
    ID(PoolS),
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
    let num = byte >= b'0' && byte <= b'9';

    return lower || upper || num;
}

impl<'a> Lexer<'a, Cursor<&[u8]>> {
    pub fn from_str(string: &'static str, sp: &'a StringPool) -> Self {
        let cursor = Cursor::new(string.as_bytes());
        let reader = BufReader::new(cursor);
        Lexer { reader, string_pool: sp }
    }
}

impl<'a, R: Read + Seek> Lexer<'a, R> {
    fn new(reader: BufReader<R>, sp: &'a StringPool) -> Self {
        Lexer { reader, string_pool: sp }
    }

    fn advance(&mut self) -> Option<u8> {
        let mut byte = [0];

        self.reader.read(&mut byte).ok().map(|read| {
            if read == 1 { Some(byte[0]) } else { None }
        }).flatten()
    }

    fn rewind(&mut self) {
        // TODO: Maybe we should implement this behavior ourselves??
        self.reader.seek_relative(-1);
    }

    fn peek(&mut self) -> Option<u8> {
        let result = self.advance();

        if result.is_some() {
            self.rewind();
        }

        result
    }

    fn matchf<F: Fn(Option<u8>) -> bool>(&mut self, f: F) -> Option<u8> {
        let next = self.advance();
        let matches = f(next);
        if matches {
            next
        }
        else {
            self.rewind();
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

    pub fn next(&mut self) -> Token {
        while self.matchf(is_whitespace).is_some() {}

        if self.peek().is_none() {
            return EOF;
        }

        // Idea...
        // Make a new function, called like, advance_predicate, or something, that returns an
        // Option<u8>, and that only advances the input stream if the character matches the predicate.
        // Otherwise, leaves the input stream untouched and returns None.
        //
        // This should allow us to completely avoid using unwrap(), as well as do less redundant work
        // otherwise.
        if let Some(mut id) = self.match_to_vec(is_alpha) {
            // while let Some(c) = self.matchf(is_alphanum) {
            //     id.push(c);
            // }
            self.match_onto_vec(&mut id, is_alphanum);

            return self.make_id(&id);
        }

        Empty
    }


}

#[cfg(test)]
mod tests {
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
}