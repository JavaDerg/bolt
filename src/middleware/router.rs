use crate::data::{Request, ResponseBuilder};
use crate::middleware::{Middleware, MiddlewareAction};
use std::sync::Arc;

pub struct Router {}

impl Middleware for Router {
    fn process<'rqs>(
        self: Arc<Self>,
        req: &'rqs mut Request,
        rb: &'rqs mut ResponseBuilder,
    ) -> MiddlewareAction<'rqs> {
        todo!()
    }
}
