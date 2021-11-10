pub mod router;

use crate::data::{Request, Response, ResponseBuilder};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub trait Middleware {
    fn process(self: Arc<Self>, req: Request, rb: &mut ResponseBuilder) -> MiddlewareAction;
}

pub enum MiddlewareAction<'s> {
    ComputeFuture(Pin<Box<dyn Future<Output = Option<Response>> + Send + Sync + 's>>),
    Direct(Option<Response>),
}

impl<'s> MiddlewareAction<'s> {
    pub async fn compute(self) -> Option<Response> {
        match self {
            MiddlewareAction::ComputeFuture(future) => future.await,
            MiddlewareAction::Direct(complete) => complete,
        }
    }
}
