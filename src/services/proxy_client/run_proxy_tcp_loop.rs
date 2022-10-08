use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use super::server_conn_manger::{ConnPair, ServerConnManger};
use tokio::net::TcpStream;

pub async fn run_proxy_tcp_loop(
    conn_manger: &ServerConnManger,
    mut ws_conn_pair: ConnPair,
    mut stream: TcpStream,
) -> Result<(), ProxyError> {
    //println!("socks5 handshake success");
    // proxy
    let proxy_result = proxy_tcp(&mut ws_conn_pair, &mut stream).await;
    let is_ws_err = match &proxy_result {
        Ok(_) => false,
        Err(e) => e.is_ws_error(),
    };
    //回收连接
    if !is_ws_err {
        conn_manger.push_back_conn(ws_conn_pair).await;
    }
    proxy_result
}
