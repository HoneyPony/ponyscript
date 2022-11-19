mod lexer;
mod string_pool;

use lexer::Lexer;
use crate::lexer::Token::EOF;
use crate::string_pool::StringPool;

fn main() {
    let sp = StringPool::new();
    let mut lex = Lexer::from_str("abc", &sp);

    loop {
        let tok = lex.next();
        dbg!(&tok);
        if tok == EOF { break; }
    }
}
