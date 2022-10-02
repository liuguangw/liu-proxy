use super::check_auth_token::check_auth_token;
use super::proxy_error::ProxyError;
use super::proxy_handshake::proxy_handshake;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async;

pub async fn handle_connection(mut stream: TcpStream, addr: SocketAddr, server_address: String) {
    println!("client {addr} connected");
    let conn_dest = match proxy_handshake(&mut stream).await {
        Ok(s) => s,
        Err(handshake_error) => {
            println!("socket5 handshake failed: {handshake_error}");
            return;
        }
    };
    let mut ws_stream = match connect_async(&server_address).await {
        Ok(s) => s.0,
        Err(e) => {
            println!("websocket handshake failed: {e}");
            return;
        }
    };
    println!("websocket handshake ok");
    //
    if let Err(auth_error) = check_auth_token(&mut ws_stream).await {
        println!("auth error: {auth_error}");
        return;
    }
    println!("server auth ok");
    //
    if let Err(proxy_error) = run_proxy_tcp_loop(&mut ws_stream, &mut stream, &conn_dest).await {
        println!("{proxy_error}");
        //断开与server之间的连接
        if !matches!(
            proxy_error,
            ProxyError::WsErr(_, _) | ProxyError::ServerClosed
        ) {
            if let Err(e1) = ws_stream.close(None).await {
                println!("close conn failed: {e1}");
            }
        }
    }
}
