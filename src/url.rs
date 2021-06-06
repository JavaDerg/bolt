use nom::AsChar;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::iter::Peekable;
use std::str::CharIndices;

trait Filter = FnMut(char) -> bool;
type CowStr<'a> = Cow<'a, str>;

#[derive(Debug)]
pub struct UrlPath<'a> {
    complete: &'a str,
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
        read_seg(&mut parser, &mut buf)?;
        loop {
            match parser.peek() {
                Some('/') => {
                    let _ = parser.next();
                    read_seg(&mut parser, &mut buf)?;
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
            segments: buf,
            query,
        })
    }
}

fn read_seg<'a>(parser: &mut Parser<'a>, buf: &mut SmallVec<[CowStr<'a>; 8]>) -> Result<(), ()> {
    let seg = parser.read_segment()?;
    match check_segment(seg.as_ref()) {
        CheckResult::Empty => (),
        CheckResult::Pop => drop(buf.pop()),
        CheckResult::Ok => buf.push(seg),
    }
    Ok(())
}

fn check_segment(seg: &str) -> CheckResult {
    if seg.is_empty() {
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
        self.next = self.pos + c.len();
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

    fn take(&mut self, mut filter: impl Filter) -> &'a str {
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
                '#' => break,
                '/' | '?' => self.next(),
                '%' => {
                    let _ = self.next();
                    let mut filter = m_lim(2, |c: char| c.is_hex_digit());
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
        Ok((!params.is_empty()).then_some(params))
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
                    let hex = self.take(m_lim(2, |c: char| c.is_hex_digit()));
                    if hex.len() != 2 {
                        return Err(());
                    }
                    buf.push(u8::from_str_radix(hex, 16).unwrap());
                }
                _ => return Err(()),
            }
            let read = self.take(m_pchar_lim);
            buf.reserve(read.len());
            for b in read.as_bytes().iter() {
                buf.push(*b);
            }
        }
        String::from_utf8(buf).map_err(|_| ())
    }
}

#[inline]
fn m_lim(len: usize, mut filter: impl Filter) -> impl Filter {
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
