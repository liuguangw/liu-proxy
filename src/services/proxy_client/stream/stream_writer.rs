use bytes::Bytes;
use std::io::{Error as IoError, ErrorKind};
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf, sync::mpsc::Sender};

pub enum StreamWriter<'a> {
    Socks5(WriteHalf<'a>),
    Http(Sender<Result<Bytes, IoError>>),
}

impl<'a> StreamWriter<'a> {
    pub async fn write_buf(&mut self, mut buf: Bytes) -> Result<(), IoError> {
        match self {
            StreamWriter::Socks5(s) => s.write_all_buf(&mut buf).await?,
            StreamWriter::Http(s) => {
                if let Err(e) = s.send(Ok(buf)).await {
                    return Err(IoError::new(ErrorKind::Other, e.to_string()));
                }
            }
        };
        Ok(())
    }
}
