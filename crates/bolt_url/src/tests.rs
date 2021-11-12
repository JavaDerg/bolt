use super::{OwnedUrlPath, UrlPath};
use std::borrow::Cow;

#[test]
fn baseline() {
    let path = UrlPath::parse("/hello/world").unwrap();
    assert_eq!(path.complete, "/hello/world");
    assert_eq!(path.sanitized_path(), "/hello/world");
    assert_eq!(path.pure, true);
    assert_eq!(
        path.segments.as_slice(),
        &[Cow::Borrowed("hello"), Cow::Borrowed("world")]
    );
}

#[test]
fn optional_slash() {
    let path = UrlPath::parse("hello/world").unwrap();
    assert_eq!(path.complete, "hello/world");
    assert_eq!(path.sanitized_path(), "/hello/world");
    assert_eq!(path.pure, false);
    assert_eq!(
        path.segments.as_slice(),
        &[Cow::Borrowed("hello"), Cow::Borrowed("world")]
    );
}

#[test]
fn hex() {
    let path = UrlPath::parse("/hello/w%C3%B6rld").unwrap();
    assert_eq!(path.complete, "/hello/w%C3%B6rld");
    assert_eq!(path.sanitized_path(), "/hello/w%C3%B6rld");
    assert_eq!(path.pure, false);
    assert_eq!(
        path.segments.as_slice(),
        &[Cow::Borrowed("hello"), Cow::Owned(String::from("wörld"))]
    );
}

#[test]
fn dots() {
    let path = UrlPath::parse("/hello/./test/../world").unwrap();
    assert_eq!(path.sanitized_path(), "/hello/world");
    assert_eq!(path.pure, false);
    assert_eq!(
        path.segments.as_slice(),
        &[Cow::Borrowed("hello"), Cow::Borrowed("world")]
    );
}

#[test]
fn query() {
    let path = UrlPath::parse("/hello/?world=hi").unwrap();
    assert_eq!(path.complete, "/hello/?world=hi");
    assert_eq!(path.sanitized_path(), "/hello?world=hi");
    assert_eq!(path.pure, false);
    assert_eq!(
        path.segments.as_slice(),
        &[Cow::Borrowed("hello")]
    );
    assert_eq!(path.query(), Some("world=hi"));
}

#[test]
fn query_hex() {
    let path = UrlPath::parse("/hello?w%C3%B6rld=hi").unwrap();
    assert_eq!(path.complete, "/hello?w%C3%B6rld=hi");
    assert_eq!(path.sanitized_path(), "/hello?w%C3%B6rld=hi");
    assert_eq!(path.pure, false);
    assert_eq!(
        path.segments.as_slice(),
        &[Cow::Borrowed("hello")]
    );
    assert_eq!(path.query(), Some("wörld=hi"));
}
