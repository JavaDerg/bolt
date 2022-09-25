use hyper::server::conn::Http;
use hyper::{Body, Request, Response};
use std::convert::Infallible;
use std::future::{ready, Ready};
use std::task::{Context, Poll};
use tokio::net::TcpListener;
use tower::Service;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    Ok(())
}
