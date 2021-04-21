use crate::responder::{FileResponder, Responder, StaticBinaryResponder};
use hyper::{Body, Request, Response};
use mlua::Function;
use std::convert::Infallible;
use std::error::Error;
use tokio::sync::Mutex;
use tokio::task::LocalSet;

pub struct Router {
    routes: Vec<(String, Box<dyn Responder + 'static + Send + Sync>)>,
    lfn: Mutex<Function<'static>>, // FIXME: This sucks x2
}

impl Router {
    pub fn new(lfn: Function<'static>) -> Self {
        Self {
            routes: vec![
                (
                    String::from("/tux"),
                    Box::new(StaticBinaryResponder {
                        data: include_bytes!("./tux.gif"),
                        content_type: "image/gif",
                    }),
                ),
                (
                    String::from("/eevee"),
                    Box::new(StaticBinaryResponder {
                        data: include_bytes!("./eevee.gif"),
                        content_type: "image/gif",
                    }),
                ),
                (
                    String::from("/test"),
                    Box::new(StaticBinaryResponder {
                        data: b"Hallo Welt!",
                        content_type: "text/plain",
                    }),
                ),
                (String::from("/"), Box::new(FileResponder)),
            ],
            lfn: Mutex::new(lfn),
        }
    }

    pub async fn route(&self, request: Request<Body>) -> Result<Response<Body>, Infallible> {
        let request_path = request.uri().path().to_string();
        let route = self
            .routes
            .iter()
            .find(|(path, ..)| request_path.starts_with(path))
            .map(|(_, responder)| responder);
        if route.is_none() {
            let table: mlua::Table = LocalSet::new()
                .run_until(async { self.lfn.lock().await.call_async(()).await.unwrap() })
                .await;
            let body = table.get::<_, Option<String>>("body")?.unwrap_or_default();
            let mut resp =
                Response::builder().status(table.get::<_, Option<u16>>("status")?.unwrap_or(200));
            if let Some(headers) = table.get::<_, Option<mlua::Table>>("headers")? {
                for pair in headers.pairs::<String, String>() {
                    let (h, v) = pair?;
                    resp = resp.header(&h, v);
                }
            }
            return Ok(resp.body(Body::from(body)).unwrap());
            return Ok(_404(&request));
        }
        let route = route.unwrap();
        Ok(match route.respond(request).await {
            Ok(response) => response,
            Err(err) => _500(err),
        })
    }
}

pub fn _404(request: &Request<Body>) -> Response<Body> {
    Response::builder()
        .status(404)
        .header("Content-Type", "text/html")
        .body(Body::from(
            maud::html! {
                (maud::DOCTYPE)
                html {
                    head { title { "404 Not Found" } }
                    body {
                        h1 { "404 Not Found" }
                        hr;
                        p { "The requested path could not be found:" }
                        code { pre { (request.uri().path()) } }
                        hr;
                        center { (env!("CARGO_PKG_NAME").to_string()) "/" (env!("CARGO_PKG_VERSION")) }
                    }
                }
            }
            .into_string(),
        ))
        .unwrap()
}

pub fn _500(error: Box<dyn Error>) -> Response<Body> {
    Response::builder()
        .status(500)
        .header("Content-Type", "text/html")
        .body(Body::from(
            maud::html! {
                (maud::DOCTYPE)
                html {
                    head { title { "500 Internal Server Error" } }
                    body {
                        h1 { "500 Internal Server Error" }
                        hr;
                        code { pre { (error) } }
                        hr;
                        center { (env!("CARGO_PKG_NAME").to_string()) "/" (env!("CARGO_PKG_VERSION")) }
                    }
                }
            }
                .into_string(),
        ))
        .unwrap()
}
