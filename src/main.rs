mod lexer;
mod string_pool;

use lexer::Lexer;
use crate::lexer::Token::EOF;
use crate::string_pool::StringPool;

fn main() {
    let sp = StringPool::new();
    let mut lex = Lexer::from_str("abc \"string literal\" lmao", &sp);

    loop {
        let tok = lex.next();
        dbg!(lex.token_to_string(&tok));
        if tok == EOF { break; }
    }
}
