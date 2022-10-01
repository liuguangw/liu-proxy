use std::{
    io::{self, ErrorKind, Result as IoResult},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use crate::common::socket5::{ConnDest,self};

pub async fn proxy_handshake(stream: &mut TcpStream) -> IoResult<ConnDest> {
    let version = stream.read_u8().await?;
    if version != socket5::VERSION {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("invalid proxy version: {version}"),
        ));
    }
    let methods = stream.read_u8().await?;
    {
        let mut buffer = vec![0; methods as usize];
        stream.read_exact(&mut buffer).await?;
        if methods==0{
            return Err(io::Error::new(
                ErrorKind::Other,
                format!("invalid methods count value: {methods}"),
            ));
        }
        //客户端不支持无密码
        if !buffer.as_slice().contains(&0){
            return Err(io::Error::new(
                ErrorKind::Other,
                "NO AUTH is not in methods".to_string(),
            ));
        }
    }
    let response_data = 0x0500;
    stream.write_u16(response_data).await?;
    //
    let version = stream.read_u8().await?;
    if version != socket5::VERSION {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("invalid proxy version: {version}"),
        ));
    }
    let cmd = stream.read_u8().await?;
    if cmd != 1 {
        return Err(io::Error::new(ErrorKind::Other, "cmd not support"));
    }
    //rsv
    stream.read_u8().await?;
     ConnDest::try_from_stream(stream).await
    .map_err(|e|io::Error::new(
        ErrorKind::Other,
        e.to_string(),
    ))
}
