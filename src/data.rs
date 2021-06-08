use crate::url::OwnedUrlPath;
use hyper::http::HeaderValue;
use hyper::{HeaderMap, Method};

pub struct Request {
    pub path: OwnedUrlPath,
    pub domain: String,
    pub header: HeaderMap<HeaderValue>,
    pub method: Method,
}

pub struct Response {
    inner: hyper::Response<hyper::Body>,
}

pub struct ResponseBuilder {
    inner: hyper::http::response::Builder,
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder {
            inner: hyper::Response::builder(),
        }
    }
}

impl ResponseBuilder {}
