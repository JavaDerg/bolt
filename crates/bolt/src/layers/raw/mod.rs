use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tower::Service;
use crate::layers::raw::stream::EitherStream;

mod stream;

pub struct UpgradeService {
    tls_acceptor: Arc<TlsAcceptor>,
}

pub struct RawRequest {
    pub stream: stream::EitherStream,
    pub secure: bool,
    pub sni_hostname: Option<String>,
    pub alpn_protocol: Option<Vec<u8>>,

    pub peer: SocketAddr,
    pub local: SocketAddr,
}

#[derive(thiserror::Error, Debug)]
pub enum UpgradeError {
    #[error("IO error while trying to upgrade to RawRequest: {0}")]
    IoError(#[from] std::io::Error),
}

impl Service<TcpStream> for UpgradeService {
    type Response = RawRequest;
    type Error = ();
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut stream: TcpStream) -> Self::Future {
        Box::pin(async move {
            let tls = check_for_tls(&mut stream).await?;

            let peer = stream.peer_addr()?;
            let local= stream.local_addr()?;

            let (stream, sni, alpn) = if tls {
                let tls_stream = self.tls_acceptor.accept(stream).await?;

                let info = tls_stream.get_ref().1;
                let sni = info.sni_hostname().map(|str| str.to_string());
                let alpn = info.alpn_protocol().map(|slice| slice.to_vec());

                (EitherStream::Tls(tls_stream), sni, alpn)
            } else {
                (EitherStream::Tcp(stream), None, None)
            };

            Ok(RawRequest {
                stream,
                secure: tls,
                sni_hostname: sni,
                alpn_protocol: alpn,
                peer,
                local,
            })
        })
    }
}

async fn check_for_tls(stream: &mut TcpStream) -> std::io::Result<bool> {
    let mut buf = [0; 1];
    stream.peek(&mut buf[..]).await?;

    Ok(buf[0] == 0x16)
}
