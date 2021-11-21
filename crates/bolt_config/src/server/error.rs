use crate::parser::lexer::Error;
use err_derive::Error;
use std::fmt::{Debug, Display, Formatter, Write};

#[derive(Debug, Error)]
#[error(display = "Error parsing the server config")]
pub enum ParseError {
    #[error(display = "Io error: {}", _0)]
    Io(#[source] std::io::Error),
    #[error(display = "Failed to lex config: {}", _0)]
    Parse(ErrorBundle),
}

pub struct ErrorBundle(pub Vec<Error>);

impl Debug for ErrorBundle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for err in &self.0 {
            Debug::fmt(err, f)?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl Display for ErrorBundle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('\n')?;
        for err in &self.0 {
            Display::fmt(err, f)?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl std::error::Error for ErrorBundle {}
