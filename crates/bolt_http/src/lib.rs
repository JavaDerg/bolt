pub mod resources;
mod inject;

use std::future::Future;
use hyper::{Body, Response};

pub struct BoltRequest {

}

#[async_trait::async_trait]
pub trait BoltResponder {
    async fn process(req: BoltRequest) {}
}

