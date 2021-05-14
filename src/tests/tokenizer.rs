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
}
