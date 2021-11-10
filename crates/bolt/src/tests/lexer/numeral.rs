use crate::config::parser::lexer::{lex, Error, ErrorKind, Token::*};

#[test]
fn single() {
    assert_eq!(
        lex("1234").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![Numeral(1234), Eof]
    );
}

#[test]
fn combined_space() {
    assert_eq!(
        lex("123 123").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![Numeral(123), Spacer, Numeral(123), Eof]
    );
}

#[test]
fn combined_complex() {
    assert_eq!(
        lex("123 u 321")
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![
            Numeral(123),
            Spacer,
            Statement("u"),
            Spacer,
            Numeral(321),
            Eof
        ]
    );
}

#[test]
fn suffix() {
    assert_eq!(
        lex("123u").map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![Numeral(123), Suffix("u"), Eof]
    );

    /* assert!(matches!(
        tokenize("123u32"),
        Err(Error {
            kind: ErrorKind::UnexpectedCharacter('3'),
            line: 0,
            pos: Range { start: 4, end: 5 },
            ..
        })
    )); */
}
