use crate::common::socket5::{self, ConnDest, ParseConnDestError};
use std::io::Error as IoError;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("invalid proxy version {0}")]
    Version(u8),
    #[error("invalid methods count value {0}")]
    Methods(u8),
    #[error("{0}")]
    IoErr(#[from] IoError),
    #[error("unsupported auth type")]
    UnsupportedAuthType,
    #[error("unsupported cmd {0}")]
    UnsupportedCmd(u8),
    #[error("parse dest failed, {0}")]
    ParseDestError(#[from] ParseConnDestError),
}

///处理socket5握手,获取目标地址、端口
pub async fn proxy_handshake(stream: &mut TcpStream) -> Result<ConnDest, HandshakeError> {
    let version = stream.read_u8().await?;
    if version != socket5::VERSION {
        return Err(HandshakeError::Version(version));
    }
    let methods = stream.read_u8().await?;
    {
        let mut buffer = vec![0; methods as usize];
        stream.read_exact(&mut buffer).await?;
        if methods == 0 {
            return Err(HandshakeError::Methods(methods));
        }
        //客户端不支持无密码
        if !buffer.as_slice().contains(&0) {
            return Err(HandshakeError::UnsupportedAuthType);
        }
    }
    let response_data = 0x0500;
    stream.write_u16(response_data).await?;
    //
    let version = stream.read_u8().await?;
    if version != socket5::VERSION {
        return Err(HandshakeError::Version(version));
    }
    let cmd = stream.read_u8().await?;
    if cmd != 1 {
        return Err(HandshakeError::UnsupportedCmd(cmd));
    }
    //rsv
    stream.read_u8().await?;
    let conn_dest = ConnDest::try_from_stream(stream).await?;
    Ok(conn_dest)
}
