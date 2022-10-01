use tokio::{net::TcpStream, io::AsyncReadExt};
use std::io::{Error as IoError, ErrorKind};

pub async fn read_raw(stream: &mut TcpStream) -> Result<Vec<u8>, IoError> {
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Err(ErrorKind::UnexpectedEof.into());
    };
    buf.truncate(n);
    Ok(buf)
}