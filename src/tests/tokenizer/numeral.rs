use crate::config::parser::tokenizer::{tokenize, Token::*, TokenizeError, TokenizeErrorKind};
use std::ops::Range;

#[test]
fn single() {
    assert_eq!(tokenize("1234"), Ok(vec![Numeral(1234), Eof]));
}

#[test]
fn combined_space() {
    assert_eq!(
        tokenize("123 123"),
        Ok(vec![Numeral(123), Spacer, Numeral(123), Eof])
    );
}

#[test]
fn combined_complex() {
    assert_eq!(
        tokenize("123 u 321"),
        Ok(vec![
            Numeral(123),
            Spacer,
            Statement("u"),
            Spacer,
            Numeral(321),
            Eof
        ])
    );
}

#[test]
fn suffix() {
    assert_eq!(tokenize("123u"), Ok(vec![Numeral(123), Suffix("u"), Eof]));

    assert!(matches!(
        tokenize("123u32"),
        Err(TokenizeError {
            kind: TokenizeErrorKind::UnexpectedCharacter('3'),
            line: 0,
            pos: Range { start: 4, end: 5 },
            ..
        })
    ));
}
