use crate::UrlPath;

#[test]
fn total_path_is_correct() {
    let path = "/foo/bar/baz".parse::<UrlPath>().unwrap();
    assert_eq!(&path.total, "/foo/bar/baz");
}

#[test]
fn empty_path() {
    let path = "".parse::<UrlPath>().unwrap();
    assert_eq!(&path.total, "");
    assert_eq!(path.parts.len(), 0);
    assert_eq!(path.query, None);
}

#[test]
fn empty_root_path() {
    let path = "/".parse::<UrlPath>().unwrap();
    assert_eq!(&path.total, "/");
    assert_eq!(path.parts.len(), 0);
    assert_eq!(path.query, None);
}

#[test]
fn basic_url_path() {
    let path = "/foo/bar/baz".parse::<UrlPath>().unwrap();
    assert_eq!(path.parts.len(), 3);
    assert_eq!(path.parts.as_slice(), &["foo", "bar", "baz"]);
    assert_eq!(path.query, None);
}

#[test]
fn no_root_path() {
    let path = "foo/bar/baz".parse::<UrlPath>().unwrap();
    assert_eq!(path.parts.len(), 3);
    assert_eq!(path.parts.as_slice(), &["foo", "bar", "baz"]);
    assert_eq!(path.query, None);
}

#[test]
fn query_url_path() {
    let path = "/foo/bar?baz=qux".parse::<UrlPath>().unwrap();
    assert_eq!(path.parts.len(), 2);
    assert_eq!(path.parts.as_slice(), &["foo", "bar"]);
    assert_eq!(path.query.as_ref().map(|r| r.as_str()), Some("baz=qux"));
}
