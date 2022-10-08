use crate::services::read_raw_data;
use bytes::{Buf, BufMut, BytesMut};
use httparse::Error as ParseError;
use std::io::Error as IoError;
use thiserror::Error;
use tokio::net::TcpStream;

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("{0}")]
    IoErr(#[from] IoError),
    #[error("{0}")]
    ParseErr(#[from] ParseError),
    #[error("not http connect request(method={1})")]
    NotConnect(&'static str, String),
}

///处理http握手,获取目标地址、端口
pub async fn proxy_handshake(
    stream: &mut TcpStream,
) -> Result<(&'static str, String), HandshakeError> {
    let mut buf = BytesMut::new();
    buf.put_slice(b"C");
    let (http_version, path) = loop {
        let data = read_raw_data::read_raw(stream).await?;
        buf.put_slice(&data);
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let res = req.parse(buf.chunk())?;
        if res.is_complete() {
            let http_version = if req.version.unwrap() == 1 {
                "1.1"
            } else {
                "1.0"
            };
            let method = req.method.unwrap();
            if method != "CONNECT" {
                return Err(HandshakeError::NotConnect(http_version, method.to_string()));
            }
            let path = req.path.unwrap().to_string();
            break (http_version, path);
        }
    };
    Ok((http_version, path))
}
