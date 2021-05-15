use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::swap;
use std::num::ParseIntError;

use nom::AsChar;
use smallvec::SmallVec;

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'a> {
    Statement(&'a str),
    Prefix(&'a str),
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
#[derive(Debug, Eq, PartialEq)]
pub enum TokenizeError {
    UnexpectedCharacter(char),
    EarlyEof,
    NotANumber,
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
    pub len_utf: usize,
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
    Prefix {
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
        len_utf: src.chars().map(|c| c.len()).sum(),
        state: ParserState::None,
        current: 0,
        cnext: 0,
        tokens: Vec::with_capacity(128),
    };
    while let Some(next) = util.next() {
        match next {
            'a'..='z' | 'A'..='Z' => {
                if util.state.is_numeral() {
                    util.submit(ParserState::Prefix {
                        start: util.current,
                        end: util.current,
                    })?;
                    continue;
                }
                if util.state.is_prefix() {
                    continue;
                }
                if !util.state.is_statement() {
                    util.submit(ParserState::Statement {
                        start: util.current,
                        end: util.current,
                    })?;
                }
            }
            c @ '0'..='9' => {
                if util.state.is_prefix() {
                    return Err(TokenizeError::UnexpectedCharacter(c));
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
                                        let c = util.next().ok_or(TokenizeError::EarlyEof)?;
                                        if !c.is_hex_digit() {
                                            return Err(TokenizeError::UnexpectedCharacter(c));
                                        }
                                    }
                                }
                                Some('u') => {
                                    // \u{2-6 hex digit}
                                    util.advance(); // u
                                    match util.next() {
                                        Some('{') => (),
                                        Some(c) => {
                                            return Err(TokenizeError::UnexpectedCharacter(c));
                                        }
                                        None => return Err(TokenizeError::EarlyEof),
                                    }
                                    for i in 0..6 {
                                        if let Some(char) = util.peek() {
                                            if !char.is_hex_digit() {
                                                if i < 2 {
                                                    return Err(
                                                        TokenizeError::UnexpectedCharacter(char),
                                                    );
                                                }
                                                break;
                                            } else {
                                                util.advance();
                                            }
                                        } else {
                                            return Err(TokenizeError::EarlyEof);
                                        }
                                    }
                                    match util.next() {
                                        Some('}') => (),
                                        Some(c) => {
                                            return Err(TokenizeError::UnexpectedCharacter(c));
                                        }
                                        None => return Err(TokenizeError::EarlyEof),
                                    }
                                }
                                Some(char) => return Err(TokenizeError::UnexpectedCharacter(char)),
                                None => return Err(TokenizeError::EarlyEof),
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
                    return Err(TokenizeError::EarlyEof);
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
                    return Err(TokenizeError::EarlyEof);
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
            _ => return Err(TokenizeError::UnexpectedCharacter(next)),
        }
    }
    util.submit(ParserState::None)?;

    util.tokens.push(Token::Eof);

    Ok(util.tokens)
}

impl Display for TokenizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TokenizeError {}

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
            ParserState::Prefix { start, end } => Token::Prefix(&src[start..end]),
            ParserState::Numeral { start, end } => Token::Numeral(
                (&src[start..end])
                    .parse()
                    .map_err(|_| TokenizeError::NotANumber)?,
            ),
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
                                            .map_err(TokenizeError::IntParseError)?
                                            as u32;
                                        last += 2;
                                        buf.push(char::from_u32(code).ok_or(
                                            TokenizeError::InvalidCharacterCode(code as u64),
                                        )?)
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
                                            .map_err(TokenizeError::IntParseError)?;
                                        buf.push(char::from_u32(code).ok_or(
                                            TokenizeError::InvalidCharacterCode(code as u64),
                                        )?);
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
            ParserState::Prefix { end, .. } => *end = pos,
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

    fn is_prefix(&self) -> bool {
        matches!(self, Self::Prefix { .. })
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
