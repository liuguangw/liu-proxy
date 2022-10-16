use super::connection::RemoteConnection;
use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use super::server_conn_manger::ServerConnManger;
use futures_util::future::Either;
use tokio::net::TcpStream;

pub async fn run_proxy_tcp_loop(
    conn_manger: &ServerConnManger,
    mut remote_conn: RemoteConnection,
    mut stream: TcpStream,
) -> Result<(), ProxyError> {
    // proxy
    let (mut remote_conn_writer, mut remote_conn_reader) = remote_conn.split();
    let proxy_result = proxy_tcp(
        &mut remote_conn_writer,
        &mut remote_conn_reader,
        &mut stream,
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
