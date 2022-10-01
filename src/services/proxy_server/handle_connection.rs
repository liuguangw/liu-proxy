use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use super::check_auth_token::check_auth_token;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use futures_util::SinkExt;

pub async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("client {addr} connected");
    let mut ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
        Ok(s) => s,
        Err(e) => {
            println!("Error during the websocket handshake occurred: {e}");
            return;
        }
    };
    if let Err(auth_error) = check_auth_token(&mut ws_stream).await {
        println!("auth error: {auth_error}");
        return;
    }
    //认证成功
    let ret_msg = Message::Binary(vec![0]);
    if let Err(ret_error) = ws_stream.send(ret_msg).await {
        println!("ret error: {ret_error}");
        return;
    }
    println!("client auth success");
    if let Err(proxy_error) = run_proxy_tcp_loop(&mut ws_stream).await{
        println!("proxy error: {proxy_error}");
    }
}
