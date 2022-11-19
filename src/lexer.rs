use std::io::{BufReader, Cursor, Read, Seek};

pub struct Lexer<R: Read + Seek> {
    reader: BufReader<R>
}

#[derive(Debug)]
#[derive(PartialEq)]

pub enum Token {
    Empty,
    ID(String),
    BadLex,
    EOF
}

impl Token {
    fn id_from(s: &str) -> Self {
        return ID(String::from(s));
    }
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

impl Lexer<Cursor<&[u8]>> {
    pub fn from_str(string: &'static str) -> Self {
        let cursor = Cursor::new(string.as_bytes());
        let reader = BufReader::new(cursor);
        Lexer { reader }
    }
}

impl<R: Read + Seek> Lexer<R> {
    fn new(reader: BufReader<R>) -> Self {
        Lexer { reader }
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
        if let Some(first) = self.matchf(is_alpha) {
            let mut id = vec![first];
            while let Some(c) = self.matchf(is_alphanum) {
                id.push(c);
            }

            let str = String::from_utf8(id);
            return match str {
                Ok(s) => ID(s),
                Err(_) => BadLex
            }
        }

        Empty
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token};

    #[test]
    fn lex_id() {
        let mut lexer = Lexer::from_str("  abc    hello    AlphaBET canhave12345 mix12and09");
        assert_eq!(lexer.next(), Token::id_from("abc"));
        assert_eq!(lexer.next(), Token::id_from("hello"));
        assert_eq!(lexer.next(), Token::id_from("AlphaBET"));
        assert_eq!(lexer.next(), Token::id_from("canhave12345"));
        assert_eq!(lexer.next(), Token::id_from("mix12and09"));
    }
}