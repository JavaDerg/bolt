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

async fn check_for_tls(stream: &mut TcpStream) -> std::io::Result<bool> {
    let mut buf = [0; 1];
    stream.peek(&mut buf[..]).await?;

    Ok(buf[0] == 0x16)
}

enum EitherStream {
    Tls(TlsStream<TcpStream>),
    Tcp(TcpStream),
}

impl AsyncRead for EitherStream {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            EitherStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
            EitherStream::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for EitherStream {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        match self.get_mut() {
            EitherStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
            EitherStream::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self.get_mut() {
            EitherStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
            EitherStream::Tcp(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self.get_mut() {
            EitherStream::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
            EitherStream::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, Error>> {
        match self.get_mut() {
            EitherStream::Tls(stream) => Pin::new(stream).poll_write_vectored(cx, bufs),
            EitherStream::Tcp(stream) => Pin::new(stream).poll_write_vectored(cx, bufs),
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        match self {
            EitherStream::Tls(stream) => Pin::new(stream).is_write_vectored(),
            EitherStream::Tcp(stream) => Pin::new(stream).is_write_vectored(),
        }
    }
}
