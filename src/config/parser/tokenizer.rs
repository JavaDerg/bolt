use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::mem::swap;
use std::num::ParseIntError;

use nom::AsChar;
use smallvec::SmallVec;
use std::iter::FromIterator;
use std::ops::Range;

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'a> {
    Statement(&'a str),
    Suffix(&'a str),
    Numeral(u64),
    String { content: Cow<'a, str>, format: bool },
    Dot,
    Equality,
    EqualitySwitch(EqualityType),
    Block(BlockType),
    Spacer,
    NewLine,
    Eof,
}

/// TODO: include location
pub struct TokenizeError {
    pub kind: TokenizeErrorKind,
    pub reason: String,
    pub line_raw: String,
    pub line: usize,
    pub pos: Range<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenizeErrorKind {
    UnexpectedCharacter(char),
    EarlyEof,
    IntParseError(ParseIntError),
    InvalidCharacterCode(u64),
}

#[derive(Debug, Eq, PartialEq)]
pub enum EqualityType {
    Equal,
    Regex,
    BeginsWith,
    EndsWith,
}

#[derive(Debug, Eq, PartialEq)]
pub enum BlockType {
    Open,
    Close,
}

struct TokenizeUtil<'a> {
    pub src: &'a str,
    pub state: ParserState,
    pub current: usize,
    pub cnext: usize,
    pub tokens: Vec<Token<'a>>,
}

enum ParserState {
    None,
    Statement {
        start: usize,
        end: usize,
    },
    Suffix {
        start: usize,
        end: usize,
    },
    Numeral {
        start: usize,
        end: usize,
    },
    String {
        start: usize,
        end: usize,
        escapes: SmallVec<[usize; 4]>,
        kind: StringType,
    },
    Dot,
    EqualitySwitch(EqualityType),
    Block(BlockType),
    Spacer,
    NewLine,
}

#[derive(Eq, PartialEq)]
enum StringType {
    SingleQuote,
    DoubleQuote,
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut util = TokenizeUtil {
        src,
        state: ParserState::None,
        current: 0,
        cnext: 0,
        tokens: Vec::with_capacity(128),
    };
    while let Some(next) = util.next() {
        match next {
            'a'..='z' | 'A'..='Z' => {
                if util.state.is_numeral() {
                    util.submit(ParserState::Suffix {
                        start: util.current,
                        end: util.current,
                    })?;
                    continue;
                }
                if util.state.is_suffix() {
                    continue;
                }
                if !util.state.is_statement() {
                    util.submit(ParserState::Statement {
                        start: util.current,
                        end: util.current,
                    })?;
                }
            }
            '0'..='9' => {
                if util.state.is_suffix() {
                    return Err(error(
                        src,
                        TokenizeErrorKind::UnexpectedCharacter(next),
                        util.current..0,
                        "A number suffix can not contain numbers",
                    ));
                }
                if !util.state.is_numeral() && !util.state.is_statement() {
                    util.submit(ParserState::Numeral {
                        start: util.current,
                        end: util.current,
                    })?;
                }
            }
            '"' => {
                let start = util.current;
                let mut escapes = SmallVec::new();
                let mut invalid = true;
                while let Some(next) = util.next() {
                    match next {
                        '\\' => {
                            escapes.push(util.current);
                            match util.peek() {
                                Some('0') | Some('n') | Some('r') | Some('t') | Some('"') => {
                                    util.advance();
                                }
                                Some('x') => {
                                    // \x00
                                    util.advance();
                                    for _ in 0..2 {
                                        let c = util.next().ok_or(error(
                                            src,
                                            TokenizeErrorKind::EarlyEof,
                                            util.current..0,
                                            "Reached EOF mid string escape",
                                        ))?;
                                        if !c.is_hex_digit() {
                                            return Err(error(
                                                src,
                                                TokenizeErrorKind::UnexpectedCharacter(c),
                                                util.current..0,
                                                "A `\\xXX` string escape code may only use a 2 digit hex numbers as value",
                                            ));
                                        }
                                    }
                                }
                                Some('u') => {
                                    // \u{2-6 hex digit}
                                    util.advance(); // u
                                    match util.next() {
                                        Some('{') => (),
                                        Some(c) => {
                                            return Err(error(
                                                src,
                                                TokenizeErrorKind::UnexpectedCharacter(c),
                                                util.current..0,
                                                "`\\u{XXXXXX}` string escapes require a `{` at the beginning of the value",
                                            ));
                                        }
                                        None => {
                                            return Err(error(
                                                src,
                                                TokenizeErrorKind::UnexpectedCharacter(next),
                                                util.current..0,
                                                "Reached EOF mid string escape",
                                            ))
                                        }
                                    }
                                    for i in 0..6 {
                                        if let Some(c) = util.peek() {
                                            if !c.is_hex_digit() {
                                                if i < 2 {
                                                    return Err(
                                                        error(
                                                            src,
                                                            TokenizeErrorKind::UnexpectedCharacter(c),
                                                            util.current..0,
                                                            "`\\u{XXXXXX}` string escapes may only use a 2-6 digit hex numbers as inner value",
                                                        ),
                                                    );
                                                }
                                                break;
                                            } else {
                                                util.advance();
                                            }
                                        } else {
                                            return Err(error(
                                                src,
                                                TokenizeErrorKind::EarlyEof,
                                                util.current..0,
                                                "Reached EOF mid string escape",
                                            ));
                                        }
                                    }
                                    match util.next() {
                                        Some('}') => (),
                                        Some(c) => {
                                            return Err(error(
                                                src,
                                                TokenizeErrorKind::UnexpectedCharacter(c),
                                                util.current..0,
                                                "`\\u{XXXXXX}` string escapes require a `}` at the end of the value",
                                            ));
                                        }
                                        None => {
                                            return Err(error(
                                                src,
                                                TokenizeErrorKind::EarlyEof,
                                                util.current..0,
                                                "Reached EOF mid string escape",
                                            ))
                                        }
                                    }
                                }
                                Some(c) => {
                                    return Err(error(
                                        src,
                                        TokenizeErrorKind::UnexpectedCharacter(c),
                                        util.current..0,
                                        format!("`\\{}` is not a known string escape", c),
                                    ))
                                }
                                None => {
                                    return Err(error(
                                        src,
                                        TokenizeErrorKind::EarlyEof,
                                        util.current..0,
                                        "Reached EOF mid string escape",
                                    ))
                                }
                            }
                        }
                        '"' => {
                            util.submit(ParserState::String {
                                start,
                                end: util.cnext,
                                escapes,
                                kind: StringType::DoubleQuote,
                            })?;
                            invalid = false;
                            break;
                        }
                        _ => {}
                    }
                }
                if invalid {
                    return Err(error(
                        src,
                        TokenizeErrorKind::EarlyEof,
                        util.current..0,
                        "Reached EOF in string",
                    ));
                }
            }
            '\'' => {
                let start = util.current;
                let mut escapes = SmallVec::new();
                let mut invalid = true;
                while let Some(char) = util.next() {
                    if char == '\'' {
                        if let Some('\'') = util.peek() {
                            escapes.push(util.current);
                            util.advance();
                            continue;
                        }
                        util.submit(ParserState::String {
                            start,
                            end: util.cnext,
                            escapes,
                            kind: StringType::SingleQuote,
                        })?;
                        invalid = false;
                        break;
                    }
                }
                if invalid {
                    return Err(error(
                        src,
                        TokenizeErrorKind::EarlyEof,
                        util.current..0,
                        "Reached EOF in string",
                    ));
                }
            }
            '.' => util.submit(ParserState::Dot)?,
            '=' | '~' | '^' | '$' => {
                util.submit(ParserState::EqualitySwitch(EqualityType::from(next)))?
            }
            '{' | '}' => util.submit(ParserState::Block(if next == '{' {
                BlockType::Open
            } else {
                BlockType::Close
            }))?,
            '\r' => {
                if let Some('\n') = util.peek() {
                    util.advance();
                }
                util.submit(ParserState::NewLine)?;
            }
            '\n' => util.submit(ParserState::NewLine)?,
            _ if next.is_whitespace() => {
                if !util.state.is_spacer() {
                    util.submit(ParserState::Spacer)?;
                }
            }
            _ => {
                return Err(error(
                    src,
                    TokenizeErrorKind::UnexpectedCharacter(next),
                    util.current..0,
                    format!("Unexpected character `{}`", next),
                ))
            }
        }
    }
    util.submit(ParserState::None)?;

    util.tokens.push(Token::Eof);

    Ok(util.tokens)
}

fn error(
    source: &str,
    kind: TokenizeErrorKind,
    mut offset: Range<usize>,
    message: impl Into<String>,
) -> TokenizeError {
    offset.end = offset.end.max(offset.start + 1);

    let mut line = 0;
    let mut lstart = 0;
    let mut citer = source
        .chars()
        .map(|c| (0, c))
        .scan(0, |counter, (_, char)| {
            let pair = (*counter, char);
            *counter += char.len();
            Some(pair)
        })
        .peekable();

    while let Some((ptr, c)) = citer.next() {
        if ptr == offset.start {
            break;
        }
        match c {
            '\r' => {
                if let Some((_, '\n')) = citer.peek() {
                    let _ = citer.next();
                    lstart = ptr + 2;
                } else {
                    lstart = ptr + 1;
                }
                line += 1;
            }
            '\n' => {
                lstart = ptr + 1;
                line += 1;
            }
            _ => (),
        }
    }
    let lend = citer
        .find(|(_, c)| *c == '\r' || *c == '\n')
        .map(|(ptr, _)| ptr)
        .unwrap_or(source.len());

    let line_raw = &source[lstart..lend];

    offset.start -= lstart;
    offset.end -= lstart;

    TokenizeError {
        kind,
        reason: message.into(),
        line_raw: String::from(line_raw),
        line,
        pos: offset,
    }
}

impl Display for TokenizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            kind: _,
            reason,
            line_raw,
            line,
            pos,
        } = self;
        writeln!(f, "[{}:{}]: {}", line, pos.start, reason)?;
        writeln!(f, "{}", line_raw)?;
        write!(f, "{}{}", "-".repeat(pos.start), "^".repeat(pos.len()))
    }
}

impl Debug for TokenizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            kind,
            reason,
            line_raw,
            line,
            pos,
        } = self;
        writeln!(f, "[{}:{}]: {} ({:?})", line, pos.start, reason, kind)?;
        writeln!(f, "{}", line_raw)?;
        write!(f, "{}{}", "-".repeat(pos.start), "^".repeat(pos.len()))
    }
}

impl Error for TokenizeError {}

impl PartialEq for TokenizeError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'a> TokenizeUtil<'a> {
    pub fn submit(&mut self, next: ParserState) -> Result<(), TokenizeError> {
        self.state.complete(self.current);
        if let Some(token) = self.state.tokenize(next, self.src)? {
            self.tokens.push(token);
        }
        Ok(())
    }

    pub fn is_available(&self) -> bool {
        self.src.len() > self.cnext
    }

    pub fn advance(&mut self) {
        let _ = self.next();
    }

    pub fn peek(&self) -> Option<char> {
        if !self.is_available() {
            None
        } else {
            self.src[self.cnext..].chars().next() // Make this nicer?
        }
    }
}

impl<'a> Iterator for TokenizeUtil<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_available() {
            self.current = self.cnext;
            None
        } else {
            let char = self.src[self.cnext..].chars().next().unwrap(); // TODO: make this nicer?
            self.current = self.cnext;
            self.cnext += char.len();
            Some(char)
        }
    }
}

impl ParserState {
    pub fn tokenize<'a>(
        &mut self,
        new: Self,
        src: &'a str,
    ) -> Result<Option<Token<'a>>, TokenizeError> {
        let old = self.swap(new);
        Ok(Some(match old {
            Self::None => return Ok(None),
            ParserState::Statement { start, end } => Token::Statement(&src[start..end]),
            ParserState::Suffix { start, end } => Token::Suffix(&src[start..end]),
            ParserState::Numeral { start, end } => {
                Token::Numeral((&src[start..end]).parse().map_err(|err| {
                    error(
                        src,
                        TokenizeErrorKind::IntParseError(err),
                        start..end,
                        format!("`{}` failed to parse this as number", &src[start..end]),
                    )
                })?)
            }
            ParserState::String {
                start,
                end,
                escapes,
                kind,
            } => {
                let str = &src[start + 1..end - 1];
                if !escapes.is_empty() {
                    Token::String {
                        content: Cow::Owned({
                            let mut buf = String::with_capacity(src.len());
                            let mut last = start + 1;
                            for escptr in escapes {
                                if last < escptr {
                                    buf.push_str(&src[last..escptr]);
                                }
                                let mut citr =
                                    (&src[escptr + 1..]).chars().zip(escptr + 1..).peekable();
                                last = escptr + 2;
                                match citr.next().unwrap() {
                                    ('0', _) => buf.push('\0'),
                                    ('n', _) => buf.push('\n'),
                                    ('r', _) => buf.push('\r'),
                                    ('t', _) => buf.push('\t'),
                                    ('"', _) => buf.push('\"'),
                                    ('\'', _) => buf.push('\''), // this also handles '' escapes in single quote strings, the pre tokenizer checks for validity
                                    ('x', ptr) => {
                                        let code = u16::from_str_radix(&src[ptr + 1..ptr + 3], 16)
                                            .map_err(|err| {
                                            error(
                                                src,
                                                TokenizeErrorKind::IntParseError(err),
                                                ptr + 1..ptr + 3,
                                                format!(
                                                    "`{}` failed to parse this as hex number",
                                                    &src[start..end]
                                                ),
                                            )
                                        })?
                                            as u32;
                                        last += 2;
                                        buf.push(char::from_u32(code).ok_or_else(|| {
                                            error(
                                                src,
                                                TokenizeErrorKind::InvalidCharacterCode(
                                                    code as u64,
                                                ),
                                                ptr + 1..ptr + 3,
                                                format!(
                                                    "`{}` is not a valid UTF-8 codepoint",
                                                    code
                                                ),
                                            )
                                        })?)
                                    }
                                    ('u', _) => {
                                        // SAFETY: Validity of syntax has been verified in pre tokenization
                                        let _ = citr.next().unwrap(); // {
                                        let (_, start) = citr.next().unwrap();
                                        let (_, stop) = (&src[start..])
                                            .chars()
                                            .zip(start..)
                                            .find(|(c, _)| *c == '}')
                                            .unwrap();
                                        last += 2 + (stop - start);
                                        let code = u32::from_str_radix(&src[start..stop], 16)
                                            .map_err(|err| {
                                                error(
                                                    src,
                                                    TokenizeErrorKind::IntParseError(err),
                                                    start..stop,
                                                    format!(
                                                        "`{}` failed to parse this as hex number",
                                                        &src[start..end]
                                                    ),
                                                )
                                            })?;
                                        buf.push(char::from_u32(code).ok_or_else(|| {
                                            error(
                                                src,
                                                TokenizeErrorKind::InvalidCharacterCode(
                                                    code as u64,
                                                ),
                                                start..stop,
                                                format!(
                                                    "`{}` is not a valid UTF-8 codepoint",
                                                    code
                                                ),
                                            )
                                        })?);
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            if end > last {
                                buf.push_str(&src[last..end - 1]);
                            }
                            buf
                        }),
                        format: matches!(kind, StringType::DoubleQuote),
                    }
                } else {
                    Token::String {
                        content: Cow::Borrowed(str),
                        format: matches!(kind, StringType::DoubleQuote),
                    }
                }
            }
            ParserState::Dot => Token::Dot,
            ParserState::EqualitySwitch(switch) => Token::EqualitySwitch(switch),
            ParserState::Block(kind) => Token::Block(kind),
            ParserState::Spacer => Token::Spacer,
            ParserState::NewLine => Token::NewLine,
        }))
    }

    pub fn complete(&mut self, pos: usize) {
        match self {
            ParserState::Statement { end, .. } => *end = pos,
            ParserState::Numeral { end, .. } => *end = pos,
            ParserState::Suffix { end, .. } => *end = pos,
            _ => (),
        }
    }

    pub fn swap(&mut self, mut new: Self) -> Self {
        swap(self, &mut new);
        new
    }

    pub fn is_spacer(&self) -> bool {
        matches!(self, Self::Spacer)
    }

    fn is_statement(&self) -> bool {
        matches!(self, Self::Statement { .. })
    }

    fn is_suffix(&self) -> bool {
        matches!(self, Self::Suffix { .. })
    }

    pub fn is_numeral(&self) -> bool {
        matches!(self, Self::Numeral { .. })
    }
}

impl From<char> for EqualityType {
    fn from(c: char) -> Self {
        match c {
            '=' => EqualityType::Equal,
            '~' => EqualityType::Regex,
            '^' => EqualityType::BeginsWith,
            '$' => EqualityType::EndsWith,
            _ => panic!("Invalid indicator char type"),
        }
    }
}
