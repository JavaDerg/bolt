use nom::AsChar;
use std::iter::Scan;
use std::str::Chars;

pub struct Tokenizer<'a> {
    string: &'a str,
    inner: Scan<Chars<'a>, usize, fn(&mut usize, char) -> Option<(usize, char)>>,
    index: usize,
    state: TokenizerState,
}

#[derive(Copy, Clone)]
enum TokenizerState {
    Normal,
    String { kind: char, escaping: bool },
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, char) = self.inner.next()?;

        match self.state {
            TokenizerState::Normal => {}
            TokenizerState::String { .. } => {}
        }

        todo!()
    }
}

impl TokenizerState {
    pub fn in_escape(&self) -> bool {
        match self {
            TokenizerState::Normal => false,
            TokenizerState::String { escaping, .. } => *escaping,
        }
    }
}

pub fn tokenize(string: &str) -> Tokenizer {
    Tokenizer {
        string,
        inner: string.chars().scan(0, |state, c| {
            let index = *state;
            *state += c.len();
            Some((index, c))
        }),
        index: 0,
        state: TokenizerState::Normal,
    }
}
