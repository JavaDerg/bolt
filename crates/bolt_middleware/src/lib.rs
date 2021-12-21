use bolt_http::{Request, Response};
use std::future::Future;
use std::mem::swap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub trait Middleware: Sync {
    fn process(self: Arc<Self>, req: Request) -> MiddlewareAction<'static>;
}

pub enum MiddlewareAction<'s> {
    ComputeFuture(Pin<Box<dyn Future<Output = Option<Response>> + Send + 's>>),
    Direct(Option<Response>),
    Depleted,
}

impl<'s> Future for MiddlewareAction<'s> {
    type Output = Option<Response>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = MiddlewareAction::Depleted;
        swap(&mut *self, &mut this);

        match this {
            MiddlewareAction::ComputeFuture(mut future) => {
                match future.as_mut().poll(cx) {
                    Poll::Ready(val) => return Poll::Ready(val),
                    Poll::Pending => (),
                }
                this = MiddlewareAction::ComputeFuture(future);
                swap(&mut *self, &mut this);
            }
            MiddlewareAction::Direct(val) => return Poll::Ready(val),
            MiddlewareAction::Depleted => panic!("Future called twice"),
        }

        Poll::Pending
    }
}
