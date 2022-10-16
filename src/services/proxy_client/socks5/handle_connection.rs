use super::{
    super::{run_proxy_tcp_loop::run_proxy_tcp_loop, server_conn_manger::ServerConnManger},
    proxy_handshake::proxy_handshake,
    write_handshake_response::write_handshake_response,
};
use crate::{
    common::RouteConfigCom,
    services::proxy_client::connection::{ConnectionError, RemoteConnection},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;

///处理socks5连接
pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    conn_manger: ServerConnManger,
    route_config: Arc<RouteConfigCom>,
) {
    //socks5初步握手,获取目标地址,端口
    let conn_dest = match proxy_handshake(&mut stream).await {
        Ok(s) => s,
        Err(handshake_error) => {
            log::error!("socks5 handshake failed [{addr}]: {handshake_error}");
            return;
        }
    };
    let conn_dest = conn_dest.to_string();
    let remote_conn = match RemoteConnection::connect(&conn_dest, &conn_manger, &route_config).await
    {
        Ok(s) => s,
        Err(e) => {
            if !matches!(e, ConnectionError::RouteBlocked) {
                log::error!("{e}");
            }
            //socket 5 通知失败信息
            if let Err(e1) = write_handshake_response(&mut stream, false).await {
                log::error!("write socks5_response failed: {e1}");
            }
            return;
        }
    };
    //socks5_response
    if let Err(e) = write_handshake_response(&mut stream, true).await {
        //回收连接
        remote_conn.push_back_conn(&conn_manger).await;
        log::error!("write socks5_response failed: {e}");
        return;
    }
    //proxy
    if let Err(proxy_error) = run_proxy_tcp_loop(&conn_manger, remote_conn, stream).await {
        log::error!("{proxy_error}");
    }
}
