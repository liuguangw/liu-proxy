use super::{
    super::{
        check_server_conn::check_server_conn, run_proxy_tcp_loop::run_proxy_tcp_loop,
        server_conn_manger::ServerConnManger,
    },
    proxy_handshake::{proxy_handshake, HandshakeError},
    write_handshake_response::write_handshake_response,
};
use std::net::SocketAddr;
use tokio::net::TcpStream;

///处理http连接
pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    conn_manger: ServerConnManger,
    first_byte: u8,
) {
    //获取目标地址,端口,http协议版本
    let (http_version, conn_dest) = match proxy_handshake(&mut stream, first_byte).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("http handshake failed [{addr}]: {e}");
            //不是http connect
            if let HandshakeError::NotConnect(http_version, _) = e {
                //http 通知失败信息
                if let Err(e1) = write_handshake_response(&mut stream, http_version, false).await {
                    log::error!("write http_response failed: {e1}");
                }
            }
            return;
        }
    };
    //向服务端发起websocket连接,并进行认证
    let mut ws_conn_pair = match conn_manger.get_conn_pair().await {
        Ok(s) => s,
        Err(e) => {
            log::error!("{e}");
            //http 通知失败信息
            if let Err(e1) = write_handshake_response(&mut stream, http_version, false).await {
                log::error!("write http_response failed: {e1}");
            }
            return;
        }
    };
    let mut is_ws_err = false;
    //把目标地址端口发给server,并检测server连接结果
    let conn_result = check_server_conn(&mut ws_conn_pair, &conn_dest).await;
    let conn_ok = conn_result.is_ok();
    match conn_result {
        Ok(_) => {
            log::info!("server conn {conn_dest} ok");
        }
        Err(e) => {
            log::error!("server conn {conn_dest} failed: {e}");
            is_ws_err = e.is_ws_error();
        }
    };
    //写入http_response
    if let Err(e) = write_handshake_response(&mut stream, http_version, conn_ok).await {
        //回收连接
        if !is_ws_err {
            conn_manger.push_back_conn(ws_conn_pair).await;
        }
        log::error!("write http_response failed: {e}");
        return;
    }
    //server连接remote失败
    if !conn_ok {
        //回收连接
        if !is_ws_err {
            conn_manger.push_back_conn(ws_conn_pair).await;
        }
        return;
    }
    //proxy
    if let Err(proxy_error) = run_proxy_tcp_loop(&conn_manger, ws_conn_pair, stream).await {
        log::error!("{proxy_error}");
    }
}
