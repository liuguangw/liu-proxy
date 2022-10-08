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
    first_byte: u8,
) -> Result<(&'static str, String), HandshakeError> {
    let mut buf = BytesMut::new();
    buf.put_u8(first_byte);
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
            let method = req.method.unwrap().to_string();
            //todo 其他请求的处理
            //> GET http://nginx.org/en/docs/http/ngx_http_proxy_module.html HTTP/1.1
            //> Host: nginx.org
            //> User-Agent: curl/7.83.1
            //> Accept: */*
            //> Proxy-Connection: Keep-Alive
            //>
            if method != "CONNECT" {
                dbg!(buf.len(), res);
                return Err(HandshakeError::NotConnect(http_version, method));
            }
            let path = req.path.unwrap().to_string();
            break (http_version, path);
        }
    };
    Ok((http_version, path))
}
