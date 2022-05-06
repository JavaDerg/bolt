use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;
use tower::{Service, ServiceExt};
use tracing::error;

mod cli;
mod layers;
mod tls;
mod util;

fn main() {
    let cpus = num_cpus::get();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(cpus)
        .build()
        .expect("Unable to build tokio runtime")
        .block_on(start());
}

async fn start() {
    if let Err(err) = listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 1337)).await {
        error!("{}", err);
    }
}

#[derive(thiserror::Error, Debug)]
enum ListenError {
    #[error("Unable to bind to {0}")]
    BindFailure(std::io::Error, SocketAddr),
    #[error("Unable to listen on {0}")]
    AcceptFailure(std::io::Error, SocketAddr),
    #[error("Io error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
enum ConnError {
    #[error("Io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Http error: {0}")]
    HttpError(#[from] hyper::Error),
}

async fn listen(addr: SocketAddr) -> Result<(), ListenError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| ListenError::BindFailure(err, addr))?;

    let config = tls::mk_config();
    let acceptor = Arc::new(TlsAcceptor::from(config.clone()));

    let mut handler = layers::raw::UpgradeService::new(acceptor);

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|err| ListenError::AcceptFailure(err, addr))?;

        let future = handler.ready().await?.call(stream);
        let jh: JoinHandle<Result<(), ConnError>> = tokio::task::spawn(async move {
            let request = future.await?;
            layers::http::RawWebService {}
                .ready()
                .await?
                .call(request)
                .await?;
            Ok(())
        });
    }
}
