mod parse;

use std::str::FromStr;

pub struct UrlPath {
    total: String,
    segments: Vec<String>,
    query: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Invalid URL")]
    NomError,
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
            segments: parts,
            query,
        })
    }
}
