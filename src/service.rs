use crate::cfg::DomainSpecificConfig;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct MainService {
    dsc: Arc<DomainSpecificConfig>,
}

impl MainService {
    pub fn new(dsc: Arc<DomainSpecificConfig>) -> Self {
        Self { dsc }
    }
}

impl Service<Request<Body>> for MainService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + 'static + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let dsc = self.dsc.clone();
        Box::pin(async move { dsc.router().route(req).await })
    }
}
