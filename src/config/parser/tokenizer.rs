use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::mem::swap;
use std::num::ParseIntError;

use smallvec::SmallVec;
use std::collections::VecDeque;
use std::ops::Range;

pub struct Tokenizer<'a> {
    src: &'a str,
    util: Util<'a>,
    state: PreState,
    backlog: VecDeque<Result<Token<'a>, Error>>,
}

enum PreState {
    Parsing,
    Eof,
    Done,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'a> {
    Statement(&'a str),
    Suffix(&'a str),
    Numeral(u64),
    String { content: Cow<'a, str>, format: bool },
    Dot,
    EqualitySwitch(EqualityType),
    Block(BlockType),
    Spacer,
    NewLine,
    Eof,
}

pub struct Error {
    pub kind: ErrorKind,
    pub reason: String,
    pub line_raw: String,
    pub line: usize,
    pub pos: Range<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    UnexpectedCharacter(char),
    EarlyEof,
    IntParseError(ParseIntError),
    InvalidCharacterCode(u64),
}

#[derive(Debug, Eq, PartialEq)]
pub enum EqualityType {
    None,
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

struct Util<'a> {
    pub src: &'a str,
    pub state: State,
    pub current: usize,
    pub cnext: usize,
}

enum State {
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

pub fn tokenize(src: &str) -> Tokenizer {
    Tokenizer {
        src,
        util: Util {
            src,
            state: State::None,
            current: 0,
            cnext: 0,
        },
        state: PreState::Parsing,
        backlog: VecDeque::with_capacity(16),
    }
}

fn error(
    source: &str,
    kind: ErrorKind,
    mut offset: Range<usize>,
    message: impl Into<String>,
) -> Error {
    offset.end = offset.end.max(offset.start + 1);

    let mut line = 0;
    let mut lstart = 0;
    let mut citer = source
        .chars()
        .map(|c| (0, c))
        .scan(0, |counter, (_, char)| {
            let pair = (*counter, char);
            *counter += char.len_utf8();
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
        .unwrap_or_else(|| source.len());

    let line_raw = &source[lstart..lend];

    offset.start -= lstart;
    offset.end -= lstart;

    Error {
        kind,
        reason: message.into(),
        line_raw: String::from(line_raw),
        line,
        pos: offset,
    }
}

macro_rules! submit {
    ($main:lifetime, $util:expr, $state:expr) => {

        match $util.submit($state) {
            Ok(Some(token)) => break $main Some(Ok(token)),
            Err(err) => break $main Some(Err(err)),
            Ok(None) => (),
        }
    };
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = 'main: loop {
            if let Some(item) = self.backlog.pop_front() {
                return Some(item);
            }
            match self.state {
                PreState::Parsing => (),
                PreState::Eof => {
                    self.state = PreState::Done;
                    return Some(Ok(Token::Eof));
                }
                PreState::Done => return None,
            }
            let next = match self.util.next() {
                Some(c) => c,
                None => {
                    self.state = PreState::Eof;
                    submit!('main, self.util, State::None);
                    continue;
                }
            };
            match next {
                'a'..='z' | 'A'..='Z' => {
                    if self.util.state.is_numeral() {
                        submit!('main,
                            self.util,
                            State::Suffix {
                                start: self.util.current,
                                end: self.util.current,
                            }
                        );
                        continue;
                    }
                    if self.util.state.is_suffix() {
                        continue;
                    }
                    if !self.util.state.is_statement() {
                        submit!('main,
                            self.util,
                            State::Statement {
                                start: self.util.current,
                                end: self.util.current,
                            }
                        );
                    }
                }
                '0'..='9' => {
                    if self.util.state.is_suffix() {
                        self.backlog.push_back(Err(error(
                            self.src,
                            ErrorKind::UnexpectedCharacter(next),
                            self.util.current..0,
                            "A number suffix can not contain numbers",
                        )));
                        continue 'main;
                    }
                    if !self.util.state.is_numeral() && !self.util.state.is_statement() {
                        submit!('main,
                            self.util,
                            State::Numeral {
                                start: self.util.current,
                                end: self.util.current,
                            }
                        );
                    }
                }
                '"' => {
                    let start = self.util.current;
                    let mut escapes = SmallVec::new();
                    let mut invalid = true;
                    'str_loop: while let Some(next) = self.util.next() {
                        match next {
                            '\\' => {
                                escapes.push(self.util.current);
                                match self.util.peek() {
                                    Some('0') | Some('n') | Some('r') | Some('t') | Some('"') => {
                                        self.util.advance();
                                    }
                                    Some('x') => {
                                        // \x00
                                        self.util.advance();
                                        for _ in 0..2 {
                                            let c = match self.util.next() {
                                                Some(c) => c,
                                                None => {
                                                    self.backlog.push_back(Err(error(
                                                        self.src,
                                                        ErrorKind::EarlyEof,
                                                        self.util.current..0,
                                                        "Reached EOF mid string escape",
                                                    )));
                                                    continue 'str_loop;
                                                }
                                            };
                                            if !c.is_ascii_hexdigit() {
                                                self.backlog.push_back(Err(error(
                                                    self.src,
                                                    ErrorKind::UnexpectedCharacter(c),
                                                    self.util.current..0,
                                                    "A `\\xXX` string escape code may only use a 2 digit hex numbers as value",
                                                )));
                                                continue 'str_loop;
                                            }
                                        }
                                    }
                                    Some('u') => {
                                        // \u{2-6 hex digit}
                                        self.util.advance(); // u
                                        match self.util.next() {
                                            Some('{') => (),
                                            Some(c) => {
                                                self.backlog.push_back(Err(error(
                                                    self.src,
                                                    ErrorKind::UnexpectedCharacter(c),
                                                    self.util.current..0,
                                                    "`\\u{XXXXXX}` string escapes require a `{` at the beginning of the value",
                                                )));
                                                continue 'str_loop;
                                            }
                                            None => {
                                                self.backlog.push_back(Err(error(
                                                    self.src,
                                                    ErrorKind::UnexpectedCharacter(next),
                                                    self.util.current..0,
                                                    "Reached EOF mid string escape",
                                                )));
                                                continue 'str_loop;
                                            }
                                        }
                                        for i in 0..6 {
                                            if let Some(c) = self.util.peek() {
                                                if !c.is_ascii_hexdigit() {
                                                    if i < 2 {
                                                        self.backlog.push_back(Err(
                                                            error(
                                                                self.src,
                                                                ErrorKind::UnexpectedCharacter(c),
                                                                self.util.current..0,
                                                                "`\\u{XXXXXX}` string escapes may only use a 2-6 digit hex numbers as inner value",
                                                            ),
                                                        ));
                                                        continue 'str_loop;
                                                    }
                                                    break;
                                                } else {
                                                    self.util.advance();
                                                }
                                            } else {
                                                self.backlog.push_back(Err(error(
                                                    self.src,
                                                    ErrorKind::EarlyEof,
                                                    self.util.current..0,
                                                    "Reached EOF mid string escape",
                                                )));
                                                continue 'str_loop;
                                            }
                                        }
                                        match self.util.next() {
                                            Some('}') => (),
                                            Some(c) => {
                                                self.backlog.push_back(Err(error(
                                                    self.src,
                                                    ErrorKind::UnexpectedCharacter(c),
                                                    self.util.current..0,
                                                    "`\\u{XXXXXX}` string escapes require a `}` at the end of the value",
                                                )));
                                                continue 'str_loop;
                                            }
                                            None => {
                                                self.backlog.push_back(Err(error(
                                                    self.src,
                                                    ErrorKind::EarlyEof,
                                                    self.util.current..0,
                                                    "Reached EOF mid string escape",
                                                )));
                                                continue 'str_loop;
                                            }
                                        }
                                    }
                                    Some(c) => {
                                        self.backlog.push_back(Err(error(
                                            self.src,
                                            ErrorKind::UnexpectedCharacter(c),
                                            self.util.current..0,
                                            format!("`\\{}` is not a known string escape", c),
                                        )));
                                        continue 'str_loop;
                                    }
                                    None => {
                                        self.backlog.push_back(Err(error(
                                            self.src,
                                            ErrorKind::EarlyEof,
                                            self.util.current..0,
                                            "Reached EOF mid string escape",
                                        )));
                                        continue 'str_loop;
                                    }
                                }
                            }
                            '\r' | '\n' => {
                                self.backlog.push_back(Err(error(
                                    self.src,
                                    ErrorKind::UnexpectedCharacter(next),
                                    self.util.current..0,
                                    "Newlines are not allowed in double quote strings, use `\n` or similar",
                                )));
                                continue 'str_loop;
                            }
                            '"' => {
                                submit!('main,
                                    self.util,
                                    State::String {
                                        start,
                                        end: self.util.cnext,
                                        escapes,
                                        kind: StringType::DoubleQuote,
                                    }
                                );
                                invalid = false;
                                break;
                            }
                            _ => {}
                        }
                    }
                    if invalid {
                        self.backlog.push_back(Err(error(
                            self.src,
                            ErrorKind::EarlyEof,
                            self.util.current..0,
                            "Reached EOF in string",
                        )));
                        continue 'main;
                    }
                }
                '\'' => {
                    let start = self.util.current;
                    let mut escapes = SmallVec::new();
                    let mut invalid = true;

                    while let Some(char) = self.util.next() {
                        if char == '\'' {
                            if let Some('\'') = self.util.peek() {
                                escapes.push(self.util.current);
                                self.util.advance();
                                continue;
                            }
                            submit!('main,
                                self.util,
                                State::String {
                                    start,
                                    end: self.util.cnext,
                                    escapes,
                                    kind: StringType::SingleQuote,
                                }
                            );
                            invalid = false;
                            break;
                        }
                    }
                    if invalid {
                        self.backlog.push_back(Err(error(
                            self.src,
                            ErrorKind::EarlyEof,
                            self.util.current..0,
                            "Reached EOF in string",
                        )));
                        continue 'main;
                    }
                }
                '.' => submit!('main, self.util, State::Dot),
                '=' | '~' | '^' | '$' | '_' => {
                    submit!('main, self.util, State::EqualitySwitch(EqualityType::from(next)))
                }
                '{' | '}' => submit!('main,
                    self.util,
                    State::Block(if next == '{' {
                        BlockType::Open
                    } else {
                        BlockType::Close
                    })
                ),
                '\r' => {
                    if let Some('\n') = self.util.peek() {
                        self.util.advance();
                    }
                    submit!('main, self.util, State::NewLine);
                }
                '\n' => submit!('main, self.util, State::NewLine),
                _ if next.is_whitespace() => {
                    if !self.util.state.is_spacer() {
                        submit!('main, self.util, State::Spacer);
                    }
                }
                _ => {
                    self.backlog.push_back(Err(error(
                        self.src,
                        ErrorKind::UnexpectedCharacter(next),
                        self.util.current..0,
                        format!("Unexpected character `{}`", next),
                    )));
                    continue 'main;
                }
            }
        };
        if !self.backlog.is_empty() {
            self.backlog.push_back(res?);
            self.backlog.pop_front()
        } else {
            res
        }
    }
}

impl Display for Error {
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

impl Debug for Error {
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

impl std::error::Error for Error {}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'a> Util<'a> {
    pub fn submit(&mut self, next: State) -> Result<Option<Token<'a>>, Error> {
        self.state.complete(self.current);
        if let Some(token) = self.state.tokenize(next, self.src)? {
            return Ok(Some(token));
        }
        Ok(None)
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

impl<'a> Iterator for Util<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_available() {
            self.current = self.cnext;
            None
        } else {
            let char = self.src[self.cnext..].chars().next().unwrap(); // TODO: make this nicer?
            self.current = self.cnext;
            self.cnext += char.len_utf8();
            Some(char)
        }
    }
}

impl State {
    pub fn tokenize<'a>(&mut self, new: Self, src: &'a str) -> Result<Option<Token<'a>>, Error> {
        let old = self.swap(new);
        Ok(Some(match old {
            Self::None => return Ok(None),
            State::Statement { start, end } => Token::Statement(&src[start..end]),
            State::Suffix { start, end } => Token::Suffix(&src[start..end]),
            State::Numeral { start, end } => {
                Token::Numeral((&src[start..end]).parse().map_err(|err| {
                    error(
                        src,
                        ErrorKind::IntParseError(err),
                        start..end,
                        format!("`{}` failed to parse this as number", &src[start..end]),
                    )
                })?)
            }
            State::String {
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
                                                ErrorKind::IntParseError(err),
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
                                                ErrorKind::InvalidCharacterCode(code as u64),
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
                                                    ErrorKind::IntParseError(err),
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
                                                ErrorKind::InvalidCharacterCode(code as u64),
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
            State::Dot => Token::Dot,
            State::EqualitySwitch(switch) => Token::EqualitySwitch(switch),
            State::Block(kind) => Token::Block(kind),
            State::Spacer => Token::Spacer,
            State::NewLine => Token::NewLine,
        }))
    }

    pub fn complete(&mut self, pos: usize) {
        match self {
            State::Statement { end, .. } => *end = pos,
            State::Numeral { end, .. } => *end = pos,
            State::Suffix { end, .. } => *end = pos,
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
            '_' => EqualityType::None,
            _ => panic!("Invalid indicator char type"),
        }
    }
}
