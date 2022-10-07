use actix_web::{error::PayloadError, web::Payload};
use bytes::{Bytes, BytesMut};
use futures_util::StreamExt;
use std::io::{Error as IoError, ErrorKind};
use tokio::{io::AsyncReadExt, net::tcp::ReadHalf};

const BUFF_SIZE: usize = 1024 * 5;
pub enum StreamReader<'a> {
    Socks5(ReadHalf<'a>),
    Http(Payload),
}

impl<'a> StreamReader<'a> {
    pub async fn read_buf(&mut self) -> Result<Bytes, IoError> {
        match self {
            Self::Socks5(s) => {
                let mut buff = BytesMut::with_capacity(BUFF_SIZE);
                let n = s.read_buf(&mut buff).await?;
                if n == 0 {
                    return Err(ErrorKind::UnexpectedEof.into());
                };
                Ok(buff.into())
            }
            Self::Http(s) => match s.next().await {
                Some(data_result) => data_result.map_err(|e| match e {
                    PayloadError::Io(e1) => e1,
                    _ => IoError::new(ErrorKind::Other, e.to_string()),
                }),
                None => Err(ErrorKind::UnexpectedEof.into()),
            },
        }
    }
}
