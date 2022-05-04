use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use hyper::{Body, Request, Response, Server};
use tower::Service;
use crate::layers::raw::RawRequest;
use crate::layers::raw::stream::EitherStream;

pub struct RawWebService {}
pub struct WebService {}

impl Service<RawRequest> for RawWebService {
    type Response = ();
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, RawRequest {
        stream, secure, sni_hostname, alpn_protocol, peer, local
    }: RawRequest) -> Self::Future {
        Box::pin(async move {
            hyper::server::conn::Http::new()
                .serve_connection(stream, WebService {})
                .await
        })
    }
}

impl Service<Request<Body>> for WebService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(async move {
            Ok(
                Response::builder()
                    .status(200)
                    .body(Body::empty())
                    .unwrap()
            )
        })
    }
}
