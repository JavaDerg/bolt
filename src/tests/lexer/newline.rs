use crate::config::parser::lexer::{lex, Token::*};

#[test]
fn line_feed() {
    assert_eq!(
        lex("\n").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![NewLine, Eof]
    );
}

#[test]
fn carriage_return() {
    assert_eq!(
        lex("\r").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![NewLine, Eof]
    );
}

#[test]
fn combined() {
    assert_eq!(
        lex("\r\n").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![NewLine, Eof]
    );
    assert_eq!(
        lex("\n\r").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![NewLine, NewLine, Eof]
    );
}
