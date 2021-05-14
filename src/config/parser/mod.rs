use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::swap;

use nom::AsChar;
use smallvec::SmallVec;

pub enum Token {}

struct ParseUtil<'a> {
    pub src: &'a str,
    pub len_utf: usize,
    pub state: ParserState,
    pub current: usize,
    pub tokens: Vec<Token>,
}

enum ParserState {
    None,
    Statement {
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
        escapes: SmallVec<[usize; 8]>,
        kind: StringType,
    },
    Seperator,
    Indicator(IndicatorType),
    Block(BlockType),
    Spacer,
    NewLine,
}

#[derive(Eq, PartialEq)]
enum IndicatorType {
    Equal,
    Regex,
    BeginsWith,
}

#[derive(Eq, PartialEq)]
enum BlockType {
    Open,
    Close,
}

#[derive(Eq, PartialEq)]
enum StringType {
    SingleQuote,
    DoubleQuote,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedCharacter(char), // TODO: include location
}

pub fn parse(src: &str) -> Result<Vec<Token>, ParseError> {
    let mut util = ParseUtil {
        src,
        len_utf: src.chars().map(|c| c.len()).sum(),
        state: ParserState::None,
        current: 0,
        tokens: Vec::with_capacity(128),
    };
    while let Some(next) = util.next() {
        match next {
            _ if util.state.is_string() => todo!(),
            'a'..='z' | 'A'..='Z' => {
                if !util.state.is_statement() {
                    util.submit(ParserState::Statement {
                        start: util.current,
                        end: util.current,
                    });
                }
            }
            '0'..='9' => {
                if !util.state.is_numeral() && !util.state.is_statement() {
                    util.submit(ParserState::Numeral {
                        start: util.current,
                        end: util.current,
                    });
                }
            }
            '"' => (),
            '\'' => (),
            '.' => util.submit(ParserState::Seperator),
            '=' | '~' | '^' => util.submit(ParserState::Indicator(IndicatorType::from(next))),
            '{' | '}' => util.submit(ParserState::Block(if next == '{' {
                BlockType::Open
            } else {
                BlockType::Close
            })),
            '\r' => {
                if let Some('\n') = util.peek() {
                    util.advance();
                }
                util.submit(ParserState::NewLine);
            }
            '\n' => util.submit(ParserState::NewLine),
            _ if next.is_whitespace() => {
                if !util.state.is_spacer() {
                    util.submit(ParserState::Spacer);
                }
            }
            _ => return Err(ParseError::UnexpectedCharacter(next)),
        }
    }

    Ok(util.tokens)
}

impl<'a> ParseUtil<'a> {
    pub fn submit(&mut self, next: ParserState) {
        self.state.complete(self.current);
        if let Some(token) = self.state.tokenize(next) {
            self.tokens.push(token);
        }
    }

    pub fn is_available(&self) -> bool {
        self.src.len() > self.current
    }

    pub fn advance(&mut self) {
        let _ = self.next();
    }

    pub fn peek(&self) -> Option<char> {
        if !self.is_available() {
            None
        } else {
            self.src[self.current..].chars().next() // Make this nicer?
        }
    }
}

impl<'a> Iterator for ParseUtil<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_available() {
            None
        } else {
            let char = self.src[self.current..].chars().next().unwrap(); // TODO: make this nicer?
            self.current += char.len();
            Some(char)
        }
    }
}

impl ParserState {
    pub fn tokenize(&mut self, new: Self) -> Option<Token> {
        let old = self.swap(new);
        match old {
            Self::None => None,
            _ => todo!(),
        }
    }

    pub fn complete(&mut self, pos: usize) {
        match self {
            ParserState::Statement { end, .. } => *end = pos,
            ParserState::Numeral { end, .. } => *end = pos,
            ParserState::String { end, .. } => *end = pos,
            _ => (),
        }
    }

    pub fn swap(&mut self, mut new: Self) -> Self {
        swap(self, &mut new);
        new
    }

    pub fn is_spacer(&self) -> bool {
        if let Self::Spacer = self {
            true
        } else {
            false
        }
    }

    fn is_statement(&self) -> bool {
        if let Self::Statement { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_numeral(&self) -> bool {
        if let Self::Numeral { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_string(&self) -> bool {
        if let Self::String { .. } = self {
            true
        } else {
            false
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ParseError {}

impl From<char> for IndicatorType {
    fn from(c: char) -> Self {
        match c {
            '=' => IndicatorType::Equal,
            '~' => IndicatorType::Regex,
            '^' => IndicatorType::BeginsWith,
            _ => panic!("Invalid indicator char type"),
        }
    }
}
