mod lexer;

use lexer::Lexer;
use crate::lexer::Token::EOF;

fn main() {
    let mut lex = Lexer::from_str("abc");

    loop {
        let tok = lex.next();
        dbg!(&tok);
        if tok == EOF { break; }
    }
}
