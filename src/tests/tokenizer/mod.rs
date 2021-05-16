use crate::config::parser::tokenizer::{tokenize, Token::*};

mod newline;
mod numeral;
mod statement;
mod string;

#[test]
fn eof() {
    assert_eq!(
        tokenize("").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![Eof]
    );
}
