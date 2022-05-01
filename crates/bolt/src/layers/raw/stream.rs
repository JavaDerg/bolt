use std::io::{Error, IoSlice};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub enum EitherStream {
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
