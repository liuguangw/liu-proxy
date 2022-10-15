use super::http::handle_connection::handle_connection as http_handle_connection;
use super::server_conn_manger::ServerConnManger;
use super::socks5::handle_connection::handle_connection as socks5_handle_connection;
use crate::common::{socks5, RouteConfigCom};
use std::{net::SocketAddr, sync::Arc};
use tokio::{io::AsyncReadExt, net::TcpStream};

///处理连接逻辑
pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    conn_manger: ServerConnManger,
    route_config: Arc<RouteConfigCom>,
) {
    let first_byte = match stream.read_u8().await {
        Ok(s) => s,
        Err(e) => {
            log::error!("handshake read data failed: {e}");
            return;
        }
    };
    if first_byte == socks5::VERSION {
        //log::info!("socks5 handshake");
        socks5_handle_connection(stream, addr, conn_manger, route_config).await;
    } else {
        //log::info!("http handshake");
        http_handle_connection(stream, addr, conn_manger, route_config, first_byte).await;
    }
}
