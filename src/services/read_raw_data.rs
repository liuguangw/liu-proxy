use bytes::{Bytes, BytesMut};
use std::io::{Error as IoError, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt};

const BUFF_SIZE: usize = 1024;
pub async fn read_raw<T>(stream: &mut T) -> Result<Bytes, IoError>
where
    T: AsyncRead + Unpin,
{
    let mut buff = BytesMut::zeroed(BUFF_SIZE);
    let n = stream.read(&mut buff).await?;
    if n == 0 {
        return Err(ErrorKind::UnexpectedEof.into());
    };
    if n < BUFF_SIZE {
        buff.truncate(n);
    }
    Ok(buff.into())
}
