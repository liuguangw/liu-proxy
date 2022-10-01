use tokio::time::{timeout, Duration};
use tokio::net::TcpStream;
use crate::common::socket5::ConnDest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnRemoteError {
    ///出错
    #[error("{0}")]
    Err(#[from] std::io::Error),
    ///超时
    #[error("connection timeout")]
    Timeout,
}

pub async fn wait_conn_remote(conn_dest:&ConnDest,timeout_duration: Duration) -> Result<TcpStream, ConnRemoteError> {
    let conn_addr = conn_dest.to_string();
    println!("server connect {conn_addr}");
    let timeout_result = timeout(timeout_duration, TcpStream::connect(&conn_addr)).await;
    match timeout_result{
        Ok(conn_result) => {
            let stream = conn_result?;
            Ok(stream)
        },
        Err(_) => Err(ConnRemoteError::Timeout),
    }
}
