use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use super::server_conn_manger::ConnPair;
use super::server_conn_manger::ServerConnManger;
use super::stream::{StreamReader, StreamWriter};

pub async fn run_proxy_tcp_loop(
    conn_manger: &ServerConnManger,
    mut ws_conn_pair: ConnPair,
    stream_reader: &mut StreamReader<'_>,
    stream_writer: &mut StreamWriter<'_>,
) -> Result<(), ProxyError> {
    //println!("socks5 handshake success");
    // proxy
    let proxy_result = proxy_tcp(&mut ws_conn_pair, stream_reader, stream_writer).await;
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
