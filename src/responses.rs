use crate::data::{Request, Response};
use std::error::Error;

pub fn _404(request: &Request) -> Response {
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

pub fn _500(error: Box<dyn Error>) -> Response {
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
