use std::error::Error;
use std::path::Path;

use async_trait::async_trait;
use hyper::{Body, Request, Response};

#[async_trait]
pub trait Responder {
    async fn respond(&self, request: Request<Body>) -> Result<Response<Body>, Box<dyn Error>>;
}

pub struct StaticBinaryResponder {
    pub data: &'static [u8],
    pub content_type: &'static str,
}

pub struct FileResponder;

#[async_trait]
impl Responder for StaticBinaryResponder {
    async fn respond(&self, _request: Request<Body>) -> Result<Response<Body>, Box<dyn Error>> {
        Ok(Response::builder()
            .header("Content-Type", self.content_type)
            .body(Body::from(self.data))?)
    }
}

#[async_trait]
impl Responder for FileResponder {
    async fn respond(&self, request: Request<Body>) -> Result<Response<Body>, Box<dyn Error>> {
        let path = todo!(); //sanitize_request_path(&request)?;
        let mut path = path.path();
        if path.starts_with('/') {
            path = &path[1..];
        }
        let mut path = Path::new("./html").join(Path::new(path));
        if path.is_dir() {
            path = path.join("index.html");
        }
        if !path.exists() {
            //return Ok(crate::router::_404(&request));
            todo!("new 404 system");
        }
        let file = tokio::fs::read(&path).await?;
        let mime = mime_guess::from_path(&path)
            .first()
            .map(|mime| mime.to_string())
            .unwrap_or_else(|| String::from("application/octet-stream"));
        Ok(Response::builder()
            .header("Content-Type", mime)
            .body(Body::from(file))?)
    }
}
