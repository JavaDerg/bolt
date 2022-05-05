use std::future::Future;
use std::pin::Pin;

pub type PinResultFuture<R, E> = Pin<Box<dyn Future<Output = Result<R, E>> + Send + 'static>>;
