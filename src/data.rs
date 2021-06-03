use std::future::Future;

pub struct Request {}
pub struct Response {}

pub trait MaybeResponse : Future<Output = Option<Response>> + Send {}
pub trait HasHeaders {}
