use std::path::Path;
use std::sync::Arc;

use async_rustls::TlsAcceptor;
use hyper::server::conn::Http;
use rustls::{KeyLog, NoClientAuth};
use tokio::net::TcpListener;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
pub use tracing::{error, info, trace, warn};
use tracing_futures::Instrument;

use crate::cfg::{DomainSpecificConfig, ServerConfig};
use crate::middleware::router::Router;
use crate::service::MainService;

mod cfg;
mod middleware;
mod service;
#[cfg(test)]
mod tests;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let url = url::UrlPath::parse("/hello");
    info!("{:#?}", url);

    info!(
        "{:#?}",
        url::UrlPath::parse("/hello/world/this/is/nice/hi%20bye/lol?hello=world#secret")
    );

    let config = cfg::ServerConfig::builder(DomainSpecificConfig::new(
        cfg::load_cert_key(Path::new("public.crt"), Path::new("private.key")),
        todo!(),
    ))
    .finish();

    let tls_config = make_ssl_config(config.clone());

    let acceptor = TlsAcceptor::from(tls_config);

    let listener = TcpListener::bind(":::8443").await.unwrap();
    // let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    /*loop {
        let (stream, peer_addr) = listener.accept().await.unwrap();

        let cfg = mod.clone();

        tokio::spawn(
            async move {
                let dsc = cfg.default();

                let service = MainService::new(dsc);

                let _ = Http::new()
                    .http1_keep_alive(true)
                    .serve_connection(stream, service)
                    .await;
            }i
            .instrument(tracing::info_span!("client", "{}", peer_addr.to_string())),
        );
    }*/

    loop {
        let (stream, peer_addr) = listener.accept().await.unwrap();
        let acceptor = acceptor.clone();

        let cfg = config.clone();

        tokio::spawn(
            async move {
                let stream = acceptor.accept(stream.compat()).await;
                if stream.is_err() {
                    warn!("Received error: {}", stream.unwrap_err());
                    return;
                }
                let stream = stream.unwrap();
                let host = stream.get_ref().1.get_sni_hostname();
                let dsc = host.map_or_else(|| cfg.default(), |host| cfg.get(host.as_bytes()));

                let service = MainService::new(dsc);

                let _ = Http::new().serve_connection(stream.compat(), service).await;
            }
            .instrument(tracing::info_span!("client", "{}", peer_addr.to_string())),
        );
    }
}

fn make_ssl_config(s_cfg: Arc<ServerConfig>) -> Arc<rustls::ServerConfig> {
    let client_auth = NoClientAuth::new();
    let mut config = rustls::ServerConfig::new(client_auth);

    config.cert_resolver = s_cfg;
    config.ticketer = rustls::Ticketer::new();
    config.set_persistence(rustls::ServerSessionMemoryCache::new(256));
    config.set_protocols(&[Vec::from(&b"http/1.1"[..]), Vec::from(&b"h2"[..])]);
    config.key_log = Arc::new(Logger);

    Arc::new(config)
}

struct Logger;

impl KeyLog for Logger {
    fn log(&self, label: &str, client_random: &[u8], secret: &[u8]) {
        trace!("{}:\n\t{:?}\n\t{:?}", label, client_random, secret);
    }
}
