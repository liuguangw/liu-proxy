use super::io::ProxyConnectResult;
use crate::common::socket5::ConnDest;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

pub async fn wait_conn_remote(
    conn_dest: &ConnDest,
    timeout_duration: Duration,
) -> ProxyConnectResult {
    let conn_addr = conn_dest.to_string();
    let timeout_result = timeout(timeout_duration, TcpStream::connect(&conn_addr)).await;
    match timeout_result {
        Ok(conn_result) => match conn_result {
            Ok(stream) => ProxyConnectResult::Ok(stream),
            Err(e) => ProxyConnectResult::Err(e),
        },
        Err(_) => ProxyConnectResult::Timeout,
    }
}
