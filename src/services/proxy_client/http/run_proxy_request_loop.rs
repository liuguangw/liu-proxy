use super::{
    super::{
        proxy_error::ProxyError,
        server_conn_manger::{ConnPair, ServerConnManger},
    },
    proxy_request::proxy_request,
};
use bytes::Bytes;
use tokio::net::TcpStream;

pub async fn run_proxy_request_loop(
    conn_manger: &ServerConnManger,
    mut ws_conn_pair: ConnPair,
    mut stream: TcpStream,
    first_request_data: Bytes,
    remain_data_size: usize,
) -> Result<(), ProxyError> {
    // proxy
    let proxy_result = proxy_request(
        &mut ws_conn_pair,
        &mut stream,
        first_request_data,
        remain_data_size,
    )
    .await;
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
