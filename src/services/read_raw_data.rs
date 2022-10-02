use std::io::{Error as IoError, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt};

pub async fn read_raw<T>(stream: &mut T) -> Result<Vec<u8>, IoError>
where
    T: AsyncRead + Unpin,
{
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Err(ErrorKind::UnexpectedEof.into());
    };
    buf.truncate(n);
    Ok(buf)
}
