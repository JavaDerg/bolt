use crate::config::parser::tokenizer::Token::*;
pub use crate::config::parser::tokenizer::{tokenize, Token};
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
fn unit_test_string_macro() {
    assert_eq!(
        string!("test", false),
        String {
            content: Cow::Borrowed("test"),
            format: false,
        }
    )
}

#[test]
fn string_basic() {
    assert_eq!(tokenize(r#"''"#), Ok(vec![string!("", false), Eof]),);
    assert_eq!(
        tokenize(r#"'Hello world'"#),
        Ok(vec![string!("Hello world", false), Eof]),
    );
    assert_eq!(
        tokenize(r#"'123 '' test'"#),
        Ok(vec![string!("123 ' test", false), Eof]),
    );
}

#[test]
fn string_advanced() {
    assert_eq!(tokenize(r#""""#), Ok(vec![string!("", true), Eof]),);
    assert_eq!(
        tokenize(r#""Hello world""#),
        Ok(vec![string!("Hello world", true), Eof]),
    );

    assert_eq!(
        tokenize(r#""123 \t test""#),
        Ok(vec![string!("123 \t test", true), Eof]),
    );

    assert_eq!(tokenize(r#""\x30""#), Ok(vec![string!("0", true), Eof]),);
    assert_eq!(
        tokenize(r#""\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39""#),
        Ok(vec![string!("0123456789", true), Eof]),
    );

    assert_eq!(
        tokenize(r#""\u{1F98A} fox; \u{1F43A} wolf""#),
        Ok(vec![string!("🦊 fox; 🐺 wolf", true), Eof]),
    );
}
