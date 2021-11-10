use crate::config::parser::lexer::{lex, Token::*};

#[test]
fn single() {
    assert_eq!(
        lex("statement")
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![Statement("statement"), Eof]
    );
}

#[test]
fn combined_space() {
    assert_eq!(
        lex("hello world")
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![Statement("hello"), Spacer, Statement("world"), Eof]
    );
}

#[test]
fn combined_dot() {
    assert_eq!(
        lex("tls.session")
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![Statement("tls"), Dot, Statement("session"), Eof]
    );
}

#[test]
fn combined_newline() {
    assert_eq!(
        lex("hi\nyou").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![Statement("hi"), NewLine, Statement("you"), Eof]
    );
}

#[test]
fn numbers() {
    assert_eq!(
        lex("hi m8\n").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![Statement("hi"), Spacer, Statement("m8"), NewLine, Eof]
    )
}
