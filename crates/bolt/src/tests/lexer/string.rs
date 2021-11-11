use crate::config::parser::lexer::{lex, Token::*};
use std::borrow::Cow;

macro_rules! string {
    ($str:literal,$fmt:ident) => {
        String {
            content: Cow::Borrowed($str),
            format: $fmt,
        }
    };
}

#[test]
fn unit_test_macro() {
    assert_eq!(
        string!("test", false),
        String {
            content: Cow::Borrowed("test"),
            format: false,
        }
    )
}

#[test]
fn basic() {
    assert_eq!(
        lex(r#"''"#).map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![string!("", false), Eof]
    );
    assert_eq!(
        lex(r#"'Hello world'"#)
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![string!("Hello world", false), Eof],
    );
    assert_eq!(
        lex(r#"'123 '' test'"#)
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![string!("123 ' test", false), Eof],
    );
}

#[test]
fn advanced() {
    assert_eq!(
        lex(r#""""#).map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![string!("", true), Eof]
    );
    assert_eq!(
        lex(r#""Hello world""#)
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![string!("Hello world", true), Eof],
    );

    assert_eq!(
        lex(r#""123 \t test""#)
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![string!("123 \t test", true), Eof],
    );

    assert_eq!(
        lex(r#""\x30""#).map(|t| t.unwrap()).collect::<Vec<_>>(),
        vec![string!("0", true), Eof],
    );
    assert_eq!(
        lex(r#""\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39""#)
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![string!("0123456789", true), Eof],
    );

    assert_eq!(
        lex(r#""\u{1F98A} fox; \u{1F43A} wolf""#)
            .map(|t| t.unwrap())
            .collect::<Vec<_>>(),
        vec![string!("🦊 fox; 🐺 wolf", true), Eof],
    );
}
