use actix_web::{http::header, HttpResponse};
use bytes::Bytes;
use futures_util::Stream;
use tokio::sync::mpsc::Receiver;

///http proxy握手时,返回的stream
pub struct ProxyResponseStream<E> {
    rx: Receiver<Result<Bytes, E>>,
}

impl<E> ProxyResponseStream<E> {
    pub fn new(rx: Receiver<Result<Bytes, E>>) -> Self {
        Self { rx }
    }
}

impl<E> Stream for ProxyResponseStream<E> {
    type Item = Result<Bytes, E>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let p = self.get_mut();
        p.rx.poll_recv(cx)
    }
}

impl<E> From<ProxyResponseStream<E>> for HttpResponse
where
    E: std::error::Error + 'static,
{
    fn from(stream: ProxyResponseStream<E>) -> Self {
        let mut resp = HttpResponse::Ok()
            .reason("Connection Established")
            .no_chunking(0)
            .streaming(stream);
        resp.headers_mut().remove(header::CONTENT_LENGTH);
        resp
    }
}
