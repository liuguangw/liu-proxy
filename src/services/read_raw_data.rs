use bytes::{Bytes, BytesMut};
use std::io::{Error as IoError, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt};

const BUFF_SIZE: usize = 1024 * 5;
pub async fn read_raw<T>(stream: &mut T) -> Result<Bytes, IoError>
where
    T: AsyncRead + Unpin,
{
    let mut buff = BytesMut::with_capacity(BUFF_SIZE);
    let n = stream.read_buf(&mut buff).await?;
    if n == 0 {
        return Err(ErrorKind::UnexpectedEof.into());
    };
    //dbg!(&buff,n);
    Ok(buff.into())
}
