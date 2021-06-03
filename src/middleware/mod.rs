use crate::data::MaybeResponse;

#[async_trait::async_trait]
pub trait Middleware {
    async fn process(&self) -> Box<dyn MaybeResponse>;
}
