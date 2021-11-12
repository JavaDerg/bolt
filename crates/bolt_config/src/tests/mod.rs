use crate::parser::lexer::{lex, Token::*};

mod newline;
mod numeral;
mod statement;
mod string;

#[test]
fn eof() {
    assert_eq!(lex("").map(|t| t.unwrap()).collect::<Vec<_>>(), vec![Eof]);
}
