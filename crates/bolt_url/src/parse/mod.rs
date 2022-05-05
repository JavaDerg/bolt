//! https://datatracker.ietf.org/doc/html/rfc3986

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::combinator::opt;
use nom::multi::{fold_many0, fold_many1, many1};
use nom::{Err, IResult};
use std::borrow::Cow;

mod hex;

/// Unnormalized!
pub fn url_path(i: &str) -> IResult<&str, (Vec<String>, Option<String>)> {
    let (i, _) = opt(tag("/"))(i)?;
    let (i, part) = opt(|i| pstr(i, false))(i)?;
    let vec = part.map(|p| vec![p]).unwrap_or_default();

    let (i, parts) = fold_many0(
        seg,
        // Sadly we can't move vec in
        || vec.clone(),
        |mut acc, str| {
            acc.push(str);
            acc
        },
    )(i)?;

    // trailing slash
    let (i, _) = opt(tag("/"))(i)?;

    let (i, query) = query(i).map(|(i, q)| (i, Some(q))).unwrap_or((i, None));

    let (i, _) = fragment(i).unwrap_or((i, ""));

    Ok((i, (parts, query)))
}

fn seg(i: &str) -> IResult<&str, String> {
    let (i, _) = tag("/")(i)?;
    pstr(i, false)
}

fn query(i: &str) -> IResult<&str, String> {
    let (i, _) = tag("?")(i)?;

    pstr(i, true)
}

fn fragment(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag("#")(i)?;

    Ok(("", i))
}

fn pstr(i: &str, query: bool) -> IResult<&str, String> {
    fold_many1(
        |i| ppiece(i, query),
        String::new,
        |mut acc, piece| {
            acc.push_str(&piece);
            acc
        },
    )(i)
}

fn ppiece(i: &str, query: bool) -> IResult<&str, Cow<str>> {
    let plain = alt((
        take_while1(unreserved),
        take_while1(sub_delims),
        take_while1(|c: char| c == ':' || c == '@'),
        take_while1(|c: char| query && (c == '/' || c == '?')),
    ))(i);
    let err = match plain {
        Ok((i, b)) => return Ok((i, Cow::Borrowed(b))),
        Err(err) => err,
    };

    match hex::take_encoded(i) {
        Ok((i, h)) => Ok((i, Cow::Owned(h))),
        Err(err) if matches!(err, Err::Failure(_)) => Err(err),
        Err(_) => Err(err),
    }
}

fn unreserved(c: char) -> bool {
    // this is technically utf8 compatible, but that should not post a problem
    c.is_alphanumeric() | matches!(c, '-' | '.' | '_' | '~')
}

fn sub_delims(c: char) -> bool {
    matches!(
        c,
        '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='
    )
}
