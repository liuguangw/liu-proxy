use std::fmt::Write;
use std::io::Result as IoResult;
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn write_handshake_response(
    stream: &mut TcpStream,
    http_version: &str,
    is_ok: bool,
) -> IoResult<()> {
    let response_text = if is_ok {
        format!("HTTP/{http_version} 200 Connection Established\r\n\r\n")
    } else {
        let message = "<h1>connect failed</h1>";
        let mut buf = format!("HTTP/{http_version} 503 Service Unavailable");
        write!(buf, "\r\nContent-Type: text/html; charset=utf-8").unwrap();
        write!(buf, "\r\nContent-length: {}", message.len()).unwrap();
        write!(buf, "\r\n\r\n{}", message).unwrap();
        buf
    };
    stream.write_all(response_text.as_bytes()).await
}
