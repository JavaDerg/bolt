pub struct Request {}
pub struct Response {
    inner: hyper::Response<hyper::Body>,
}

pub struct ResponseBuilder;

pub trait HasHeaders {}
