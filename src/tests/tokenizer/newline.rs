use crate::config::parser::tokenizer::{tokenize, Token::*};

#[test]
fn line_feed() {
    assert_eq!(
        tokenize("\n").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![NewLine, Eof]
    );
}

#[test]
fn carriage_return() {
    assert_eq!(
        tokenize("\r").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![NewLine, Eof]
    );
}

#[test]
fn combined() {
    assert_eq!(
        tokenize("\r\n").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![NewLine, Eof]
    );
    assert_eq!(
        tokenize("\n\r").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![NewLine, NewLine, Eof]
    );
}
