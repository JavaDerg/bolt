use crate::layers::raw::stream::EitherStream;
use crate::util::PinResultFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tower::Service;

pub mod stream;

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

impl UpgradeService {
    pub fn new(acceptor: Arc<TlsAcceptor>) -> Self {
        Self {
            tls_acceptor: acceptor,
        }
    }
}

impl Service<TcpStream> for UpgradeService {
    type Response = RawRequest;
    type Error = std::io::Error;
    type Future = PinResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut stream: TcpStream) -> Self::Future {
        let acceptor = self.tls_acceptor.clone();

        Box::pin(async move {
            let tls = check_for_tls(&mut stream).await?;

            let peer = stream.peer_addr()?;
            let local = stream.local_addr()?;

            let (stream, sni, alpn) = if tls {
                let tls_stream = acceptor.accept(stream).await?;

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
