use crate::services::proxy_client::connection::RemoteConnection;

use super::{
    super::{proxy_error::ProxyError, server_conn_manger::ServerConnManger},
    proxy_request::proxy_request,
};
use bytes::Bytes;
use futures_util::future::Either;
use tokio::net::TcpStream;

pub async fn run_proxy_request_loop(
    conn_manger: &ServerConnManger,
    mut remote_conn: RemoteConnection,
    mut stream: TcpStream,
    first_request_data: Bytes,
    remain_data_size: usize,
) -> Result<(), ProxyError> {
    // proxy
    let (mut remote_conn_writer, mut remote_conn_reader) = remote_conn.split();
    let proxy_result = proxy_request(
        &mut remote_conn_writer,
        &mut remote_conn_reader,
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
        if let Either::Right(writer) = remote_conn_writer.inner_writer {
            if let Either::Right(reader) = remote_conn_reader.inner_reader {
                let ws_conn_pair = (writer, reader);
                conn_manger.push_back_conn(ws_conn_pair).await;
            }
        }
    }
    proxy_result
}
