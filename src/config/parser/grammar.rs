use crate::config::parser::tokenizer::{BlockType, EqualityType, Token};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::iter::Peekable;

pub struct GrammarIter<'a, I: Iterator<Item = Token<'a>>> {
    inner: Peekable<I>,
}

#[derive(Debug)]
pub enum Item<'a> {
    Command(Command<'a>),
}

#[derive(Debug)]
pub struct Command<'a>(Key<'a>, SmallVec<[Value<'a>; 4]>, Option<Box<Block<'a>>>);
#[derive(Debug)]
pub struct Key<'a>(SmallVec<[&'a str; 4]>);
#[derive(Debug)]
pub struct Block<'a>(SmallVec<[Item<'a>; 8]>);
#[derive(Debug)]
pub enum Value<'a> {
    Statement(&'a str),
    String { content: Cow<'a, str>, format: bool },
    Number { value: u64, suffix: Option<&'a str> },
    Equator(EqualityType),
}

pub fn process_semantics<'a, I: Iterator<Item = Token<'a>>>(iter: I) -> GrammarIter<'a, I> {
    GrammarIter {
        inner: iter.peekable(),
    }
}

impl<'a, I: Iterator<Item = Token<'a>>> GrammarIter<'a, I> {
    fn read_key(&mut self) -> Option<Key<'a>> {
        let mut vec = SmallVec::new();
        loop {
            // It is ok to use `?` here as we can't return anything if there is nothing left
            match self.inner.peek()? {
                Token::Spacer | Token::NewLine => drop(self.inner.next()),
                Token::Eof => return None,
                _ => break,
            }
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

    fn priv_next(&mut self, in_block: bool) -> Option<Item<'a>> {
        loop {
            let key = self.read_key()?;
            let mut values = SmallVec::new();
            if !matches!(self.inner.next()?, Token::Spacer) {
                return None;
            }
            loop {
                match self.inner.next() {
                    Some(Token::Spacer) => continue,
                    Some(Token::NewLine) => return Some(Item::Command(Command(key, values, None))),
                    Some(Token::Statement(str)) => values.push(Value::Statement(str)),
                    Some(Token::String { content, format }) => {
                        values.push(Value::String { content, format })
                    }
                    Some(Token::Numeral(num)) => {
                        if let Some(Token::Suffix(_)) = self.inner.peek() {
                            values.push(Value::Number {
                                value: num,
                                suffix: self.inner.next().map(|i| match i {
                                    Token::Suffix(sfx) => sfx,
                                    _ => unreachable!(),
                                }),
                            })
                        } else {
                            values.push(Value::Number {
                                value: num,
                                suffix: None,
                            });
                        }
                    }
                    Some(Token::EqualitySwitch(etype)) => values.push(Value::Equator(etype)),
                    Some(Token::Eof) | None => return None,
                    Some(Token::Block(BlockType::Open)) => {
                        while let Some(Token::Spacer) = self.inner.peek() {
                            let _ = self.inner.next();
                        }
                        if !matches!(self.inner.next()?, Token::NewLine) {
                            return None; // error
                        }
                        let mut vec = SmallVec::new();
                        while let Some(item) = self.priv_next(true) {
                            vec.push(item);
                        }
                        return Some(Item::Command(Command(
                            key,
                            values,
                            Some(Box::new(Block(vec))),
                        )));
                    }
                    Some(Token::Block(BlockType::Close)) => {
                        if in_block {
                            return None;
                        } else {
                            panic!("make this a proper error");
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }
}

impl<'a, I: Iterator<Item = Token<'a>>> Iterator for GrammarIter<'a, I> {
    type Item = Item<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.priv_next(false)
    }
}
