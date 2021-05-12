use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::swap;

use smallvec::SmallVec;

pub enum Token {}

struct ParseUtil<'a> {
    pub src: &'a str,
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
    Spacer,
    NewLine,
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
            '0'..='9' => (),
            '"' => (),
            '\'' => (),
            '\\' => (),
            '.' => (),
            '{' => (),
            '}' => (),
            '\r' => (),
            '\n' => (),
            _ if next.is_whitespace() => (),
            _ => ParseError::UnexpectedCharacter(next),
        }
    }

    Ok(util.tokens)
}

impl ParseUtil {
    pub fn submit(&mut self, next: ParserState) {
        self.state.complete(self.current);
        self.tokens.push(self.state.tokenize(next));
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
            Some(self.src[self.current])
        }
    }
}

impl Iterator for ParseUtil {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_available() {
            None
        } else {
            let char = self.src[self.current];
            self.current += 1;
            Some(char)
        }
    }
}

impl ParserState {
    pub fn tokenize(&mut self, new: Self) -> Token {
        let old = self.swap(new);
        match old {
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

    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
    }

    pub fn is_spacer(&self) -> bool {
        if let Self::Spacer = self {
            true
        } else {
            false
        }
    }

    pub fn is_newline(&self) -> bool {
        if let Self::NewLine = self {
            true
        } else {
            false
        }
    }

    pub fn is_statement(&self) -> bool {
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
