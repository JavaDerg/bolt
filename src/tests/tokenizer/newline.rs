use crate::config::parser::tokenizer::{tokenize, Token::*};

#[test]
fn line_feed() {
    assert_eq!(tokenize("\n"), Ok(vec![NewLine, Eof]));
}

#[test]
fn carriage_return() {
    assert_eq!(tokenize("\r"), Ok(vec![NewLine, Eof]));
}

#[test]
fn combined() {
    assert_eq!(tokenize("\r\n"), Ok(vec![NewLine, Eof]));
    assert_eq!(tokenize("\n\r"), Ok(vec![NewLine, NewLine, Eof]));
}
