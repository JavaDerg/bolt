mod parse;
#[cfg(test)]
mod tests;

use std::str::FromStr;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct UrlPath {
    pub(crate) total: String,
    pub(crate) parts: Vec<String>,
    pub(crate) query: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Invalid URL")]
    NomError,
}

impl UrlPath {
    pub fn total(&self) -> &str {
        &self.total
    }

    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }
}

impl FromStr for UrlPath {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, (parts, query)) = parse::url_path(s).map_err(|_| ParseError::NomError)?;

        if !left.is_empty() {
            return Err(ParseError::NomError);
        }

        Ok(UrlPath {
            total: s.to_string(),
            parts: parts.into_iter().map(|s| normalize_str(&s)).collect(),
            query: query.map(|s| normalize_str(&s)),
        })
    }
}

pub fn normalize_str(s: &str) -> String {
    s.nfc().collect::<String>()
}
