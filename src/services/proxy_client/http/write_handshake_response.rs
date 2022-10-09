use std::fmt::Write;
use std::io::Result as IoResult;
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn write_handshake_response(
    stream: &mut TcpStream,
    http_version: &str,
    req_path: &str,
    is_ok: bool,
) -> IoResult<()> {
    let response_text = if is_ok {
        format!("HTTP/{http_version} 200 Connection Established\r\n\r\n")
    } else {
        let message = format!("<h1>request failed</h1><p>request {req_path} failed</p>");
        let mut buf = format!("HTTP/{http_version} 503 Service Unavailable");
        write!(buf, "\r\nContent-Type: text/html; charset=utf-8").unwrap();
        write!(buf, "\r\nContent-length: {}", message.len()).unwrap();
        write!(buf, "\r\n\r\n{message}").unwrap();
        buf
    };
    stream.write_all(response_text.as_bytes()).await
}
