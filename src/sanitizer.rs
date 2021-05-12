use std::error::Error;
use std::fmt::{Display, Formatter};

use hyper::{Body, Request};
use url::Url;

#[derive(Debug)]
pub struct SanitizationError;

pub fn sanitize_request_path(request: &Request<Body>) -> Result<Url, SanitizationError> {
    Url::parse(&format!(
        "http://localhost{}",
        request
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("")
    ))
        .map_err(|_| SanitizationError)
}

impl Display for SanitizationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to sanitize url")
    }
}

impl Error for SanitizationError {}
