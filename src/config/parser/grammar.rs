use crate::config::parser::tokenizer::{EqualityType, Token};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::iter::Peekable;

pub struct GrammarIter<'a, I: Iterator<Item = Token<'a>>> {
    inner: Peekable<I>,
}

pub enum Item<'a> {
    Command(Command<'a>),
}

pub struct Command<'a>(Key<'a>, Value<'a>, Option<Box<Block<'a>>>);
pub struct Key<'a>(SmallVec<[&'a str; 4]>);
pub struct Block<'a>(SmallVec<[Item<'a>; 8]>);
pub enum Value<'a> {
    Statement(&'a str),
    String { content: Cow<'a, str>, format: bool },
    Number { value: u64, prefix: Option<&'a str> },
    Equator(EqualityType),
}

impl<'a, I: Iterator<Item = Token<'a>>> GrammarIter<'a, I> {
    pub fn read_key(&mut self) -> Option<Key<'a>> {
        let mut vec = SmallVec::new();
        while let Some(Token::Spacer) = self.inner.peek() {
            let _ = self.inner.next();
        }
        loop {
            if let Token::Statement(str) = self.inner.next()? {
                vec.push(str);
            }
            if let Some(Token::Dot) = self.inner.peek() {
                continue;
            } else {
                break;
            }
        }
        Some(Key(vec))
    }
}

impl<'a, I: Iterator<Item = Token<'a>>> Iterator for GrammarIter<'a, I> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;

        loop {
            let key = self.read_key()?;
            loop {
                match self.inner.peek() {
                    Some(Token::Spacer) => todo!(),
                    Some(Token::NewLine) => todo!(),
                    _ => todo!(),
                }
            }
        }

        todo!()
    }
}

struct Wrapper<'a, I: Iterator<Item = &'a str>>(I);

impl<'a, I: Iterator<Item = &'a str>> Iterator for Wrapper<'a, I> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let something = self.something()?;
        loop {
            let other = self.0.next();
        }
    }
}

impl<'a, I: Iterator<Item = &'a str>> Wrapper<'a, I> {
    fn something(&mut self) -> Option<&'a str> {
        let next = self.0.next()?;
        Some(if next.starts_with("_") {
            &next[1..]
        } else {
            next
        })
    }
}
