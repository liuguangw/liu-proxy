use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsStream;

///stream封装
pub enum ServerStream<S> {
    ///原始流
    Plain(S),
    ///加密流
    Rustls(TlsStream<S>),
}

impl<S> AsyncRead for ServerStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            ServerStream::Plain(s) => Pin::new(s).poll_read(cx, buf),
            ServerStream::Rustls(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl<S> AsyncWrite for ServerStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            ServerStream::Plain(s) => Pin::new(s).poll_write(cx, buf),
            ServerStream::Rustls(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            ServerStream::Plain(s) => Pin::new(s).poll_flush(cx),
            ServerStream::Rustls(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            ServerStream::Plain(s) => Pin::new(s).poll_shutdown(cx),
            ServerStream::Rustls(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}
