// TODO: Add tests!!!

///! implemented according to https://www.ietf.org/rfc/rfc3986.txt

#[cfg(test)]
mod tests;

use once_cell::sync::OnceCell;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::iter::Peekable;
use std::marker::PhantomPinned;
use std::ops::{Deref, Not};
use std::pin::Pin;
use std::str::CharIndices;

type CowStr<'a> = Cow<'a, str>;

pub struct OwnedUrlPath(Pin<Box<InnerOwnedUrlPath>>);

struct InnerOwnedUrlPath {
    path: String,
    parts: Option<UrlPath<'static>>,
    _pin: PhantomPinned,
}

#[derive(Debug)]
pub struct UrlPath<'a> {
    complete: &'a str,
    sanitized: OnceCell<CowStr<'a>>,
    pure: bool,
    segments: SmallVec<[CowStr<'a>; 8]>,
    query: Option<&'a str>,
}

struct Parser<'a> {
    data: &'a str,
    iter: Peekable<CharIndices<'a>>,
    pos: usize,
    next: usize,
}

enum CheckResult {
    Empty,
    Pop,
    Ok,
}

impl OwnedUrlPath {
    pub fn new(path: impl Into<String>) -> Result<Self, ()> {
        let mut boxed = Box::pin(InnerOwnedUrlPath {
            path: path.into(),
            parts: None,
            _pin: PhantomPinned,
        });

        // TODO: check if this is UB
        let paths = unsafe { std::mem::transmute(UrlPath::parse(&boxed.path)?) };
        unsafe {
            let mut_ref: Pin<&mut InnerOwnedUrlPath> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).parts = Some(paths);
        }
        Ok(Self(boxed))
    }

    pub fn inner(&self) -> &UrlPath {
        // TODO: check if this is UB
        unsafe { std::mem::transmute(self.0.parts.as_ref().unwrap()) }
    }
}

impl<'a> UrlPath<'a> {
    pub fn parse(url: &'a str) -> Result<UrlPath<'a>, ()> {
        let mut parser = Parser {
            data: url,
            iter: url.char_indices().peekable(),
            pos: 0,
            next: 0,
        };
        let mut buf = SmallVec::<[CowStr<'a>; 8]>::new();
        let mut query = None;

        parser.optional('/');
        let mut pure = read_seg(&mut parser, &mut buf)?;
        loop {
            match parser.peek() {
                Some('/') => {
                    let _ = parser.next();
                    pure &= read_seg(&mut parser, &mut buf)?;
                }
                Some('?') => {
                    let _ = parser.next();
                    query = parser.take_query()?;
                    break;
                }
                Some('#') | None => break,
                _ => return Err(()),
            }
        }

        Ok(Self {
            complete: url,
            sanitized: OnceCell::new(),
            pure,
            segments: buf,
            query,
        })
    }

    pub fn original_path(&self) -> &str {
        self.complete
    }

    pub fn sanitized_path(&self) -> &str {
        self.sanitized
            .get_or_init(|| {
                self.pure
                    .then(|| Cow::Borrowed(self.complete))
                    .unwrap_or_else(|| Cow::Owned(self.sanitized()))
            })
            .as_ref()
    }

    pub fn segments(&self) -> &[CowStr] {
        self.segments.as_slice()
    }

    pub fn query(&self) -> Option<&str> {
        self.query
    }

    fn sanitized(&self) -> String {
        let mut buffer = String::new();
        self.segments.iter().for_each(|str| {
            buffer.push('/');
            if let CowStr::Borrowed(borrow) = str {
                buffer.push_str(*borrow);
                return;
            }
            let mut parser = Parser {
                data: str.as_ref(),
                iter: str.char_indices().peekable(),
                pos: 0,
                next: 0,
            };
            loop {
                let str = parser.take(m_pchar_lim);
                if str.is_empty() && parser.peek().is_none() {
                    break;
                } else if str.is_empty() {
                    let char = parser.next().unwrap();
                    let len = char.len_utf8();
                    buffer.reserve(len * 2 + 1);
                    buffer.push('%');
                    let char = char as u32;
                    for i in (0..len).rev() {
                        let byte = (char >> i * 8 & 0xFF) as u8;
                        let (b1, b2) = (byte >> 4, byte & 0xF);
                        buffer.push(as_hex_digit(b1));
                        buffer.push(as_hex_digit(b2));
                    }
                    continue;
                }
                buffer.push_str(str);
            }
        });
        buffer
    }
}

fn as_hex_digit(b: u8) -> char {
    match b {
        0..=9 => (b'0' + b) as char,
        10..=15 => (b'A' + b) as char,
        _ => panic!("Out of range"),
    }
}

fn read_seg<'a>(parser: &mut Parser<'a>, buf: &mut SmallVec<[CowStr<'a>; 8]>) -> Result<bool, ()> {
    let seg = parser.read_segment()?;
    let borrowed = matches!(&seg, Cow::Borrowed(_));
    let pure = match check_segment(seg.as_ref()) {
        CheckResult::Empty => false,
        CheckResult::Pop => {
            let _ = buf.pop();
            false
        }
        CheckResult::Ok => {
            buf.push(seg);
            true
        }
    };
    Ok(pure && borrowed)
}

fn check_segment(seg: &str) -> CheckResult {
    if seg.is_empty() || seg == "." {
        CheckResult::Empty
    } else if seg == ".." {
        CheckResult::Pop
    } else {
        CheckResult::Ok
    }
}

impl<'a> Parser<'a> {
    fn next(&mut self) -> Option<char> {
        let (i, c) = self.iter.next()?;
        self.pos = i;
        self.next = self.pos + c.len_utf8();
        Some(c)
    }

    fn peek(&mut self) -> Option<char> {
        self.iter.peek().map(|(_, c)| *c)
    }

    fn optional(&mut self, c: char) {
        if self.peek() == Some(c) {
            let _ = self.next();
        }
    }

    fn take(&mut self, mut filter: impl FnMut(char) -> bool) -> &'a str {
        let start = self.next;
        while let Some(c) = self.peek() {
            if !filter(c) {
                break;
            }
            let _ = self.next();
        }
        &self.data[start..self.next]
    }

    fn take_query(&mut self) -> Result<Option<&'a str>, ()> {
        let start = self.next;
        while let Some(c) = self.peek() {
            let _ = match c {
                // ignore hash, it's not meant for the client
                '#' => break,
                '/' | '?' => self.next(),
                '%' => {
                    let _ = self.next();
                    let mut filter = m_lim(2, |c: char| c.is_ascii_hexdigit());
                    let rs = self.pos;
                    while filter(self.peek().ok_or(())?) {
                        let _ = self.next();
                    }
                    if self.pos - rs != 2 {
                        return Err(());
                    }
                    continue;
                }
                _ if m_pchar_lim(c) => self.next(),
                _ => return Err(()),
            };
        }
        let params = &self.data[start..self.next];
        Ok(match params.is_empty().not() {
            true => Some(params),
            false => None,
        })
    }

    fn read_segment(&mut self) -> Result<CowStr<'a>, ()> {
        let read = self.take(m_pchar_lim);
        let p = self.peek();
        match p {
            Some('/' | '?' | '#') | None => Ok(Cow::Borrowed(read)),
            Some('%') => Ok(Cow::Owned(self.continue_owned(read)?)),
            _ => Err(()),
        }
    }

    fn continue_owned(&mut self, start: &str) -> Result<String, ()> {
        let mut buf = Vec::from(start);
        while let Some(c) = self.peek() {
            match c {
                '/' | '?' | '#' => break,
                '%' => {
                    let _ = self.next();
                    let hex = self.take(m_lim(2, |c: char| c.is_ascii_hexdigit()));
                    if hex.len() != 2 {
                        return Err(());
                    }
                    buf.push(u8::from_str_radix(hex, 16).unwrap());
                }
                _ => return Err(()),
            }
            let read = self.take(m_pchar_lim);
            if !read.is_empty() {
                buf.reserve(read.len());
                for b in read.as_bytes().iter() {
                    buf.push(*b);
                }
            }
        }
        String::from_utf8(buf).map_err(|_| ())
    }
}

#[inline]
fn m_lim(len: usize, mut filter: impl FnMut(char) -> bool) -> impl FnMut(char) -> bool {
    let mut count = 0;
    move |c| {
        if count >= len {
            false
        } else {
            count += 1;
            filter(c)
        }
    }
}

#[inline]
fn m_pchar_lim(c: char) -> bool {
    match c {
        _ if m_unreserved(c) | m_sub_delims(c) => true,
        ':' | '@' => true,
        _ => false,
    }
}

#[inline]
fn m_unreserved(c: char) -> bool {
    match c {
        _ if c.is_ascii_alphanumeric() => true,
        '-' | '.' | '_' | '~' => true,
        _ => false,
    }
}

#[inline]
fn m_sub_delims(c: char) -> bool {
    matches!(
        c,
        '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='
    )
}

/*
    pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"

    unreserved    = ALPHA / DIGIT / "-" / "." / "_" / "~"
    pct-encoded   = "%" HEXDIG HEXDIG
    sub-delims    = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
*/
