use hyper::{Body, Request};
use std::future::Pending;
use std::task::{Context, Poll};
use tower::Service;

pub struct DynService {}

impl Service<Request<Body>> for DynService {
    type Response = ();
    type Error = ();
    type Future = Pending<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        todo!()
    }
}
