use crate::data::Response;
use std::future::Future;

pub trait Middleware {
    fn process(&self) -> MiddlewareAction<'_>;
}

pub enum MiddlewareAction<'s> {
    ComputeFuture(Box<dyn Future<Output = Option<Response>> + Send + 's>),
    Direct(Option<Response>),
}
