use std::io::{Error, IoSlice};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

mod cli;
mod tls;
mod layers;

fn main() {
    let cpus = num_cpus::get();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(cpus)
        .build()
        .expect("Unable to build tokio runtime")
        .block_on(start());
}

async fn start() {}

#[derive(thiserror::Error, Debug)]
enum ListenError {
    #[error("Unable to bind to {0}")]
    BindFailure(std::io::Error, SocketAddr),
    #[error("Unable to listen on {0}")]
    AcceptFailure(std::io::Error, SocketAddr),
}

async fn listen(addr: SocketAddr) -> Result<(), ListenError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| ListenError::BindFailure(err, addr))?;

    let config = tls::mk_config();
    let acceptor = Arc::new(TlsAcceptor::from(config.clone()));

    loop {
        let (mut stream, addr) = listener
            .accept()
            .await
            .map_err(|err| ListenError::AcceptFailure(err, addr))?;

        let acceptor = acceptor.clone();
        let jh: JoinHandle<std::io::Result<()>> = tokio::spawn(async move {
            let tls = check_for_tls(&mut stream).await?;

            let stream = if tls {
                EitherStream::Tls(acceptor.accept(stream).await?)
            } else {
                EitherStream::Tcp(stream)
            };

            Ok(())
        });
    }

    Ok(())
}
