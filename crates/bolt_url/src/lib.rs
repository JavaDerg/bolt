#[cfg(test)]
mod tests;

mod border;
///! implemented according to https://www.ietf.org/rfc/rfc3986.txt
pub mod old;
mod slash;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, Match};
use std::borrow::Cow;
use std::marker::PhantomPinned;
use std::pin::Pin;

type CowStr<'a> = Cow<'a, str>;

pub struct OwnedUrlPath(Pin<Box<InnerOwnedUrlPath>>);
struct InnerOwnedUrlPath {
    path: String,
    parts: Option<UrlPath<'static>>,
    _pin: PhantomPinned,
}

pub struct UrlPath<'a> {
    complete: &'a str,
}

/*
    pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"

    unreserved    = ALPHA / DIGIT / "-" / "." / "_" / "~"
    pct-encoded   = "%" HEXDIG HEXDIG
    sub-delims    = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
*/
