use super::proxy_handshake::proxy_handshake;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use crate::{
    common::{socket5::build_response, WebsocketRequest},
    services::proxy_client::auth_handshake,
};
use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::TcpStream};

///处理连接逻辑
pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    ws_request: WebsocketRequest,
) {
    //socket5初步握手,获取目标地址,端口
    let conn_dest = match proxy_handshake(&mut stream).await {
        Ok(s) => s,
        Err(handshake_error) => {
            log::error!("socket5 handshake failed [{addr}]: {handshake_error}");
            return;
        }
    };
    //向服务端发起websocket连接,并进行认证
    let ws_stream = match auth_handshake(&ws_request).await {
        Ok(s) => s.0,
        Err(e) => {
            log::error!("{e}");
            //socket 5 通知失败信息
            let rep_code = 5;
            let socket5_response = build_response(&conn_dest, rep_code);
            if let Err(e1) = stream.write_all(&socket5_response).await {
                log::error!("write socket5_response failed: {e1}");
            }
            return;
        }
    };
    if let Err(proxy_error) = run_proxy_tcp_loop(ws_stream, &mut stream, &conn_dest).await {
        log::error!("{proxy_error}");
    }
}
