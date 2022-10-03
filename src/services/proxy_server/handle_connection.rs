use super::check_auth_token::check_auth_token;
use super::proxy_error::ProxyError;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use crate::common::{ServerConfig, ServerStream};
use futures_util::SinkExt;
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::tungstenite::Message;

///处理连接逻辑
pub async fn handle_connection(
    raw_stream: TcpStream,
    peer_addr: SocketAddr,
    config: ServerConfig,
    tls_acceptor: Option<TlsAcceptor>,
) {
    //println!("client {peer_addr} connected");
    //处理tls握手
    let stream = match tls_acceptor {
        Some(acceptor) => {
            let tls_stream = match acceptor.accept(raw_stream).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("accept tls client {peer_addr} failed: {e}");
                    return;
                }
            };
            ServerStream::Rustls(tls_stream.into())
        }
        None => ServerStream::Plain(raw_stream),
    };
    //处理websocket握手
    let mut ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("accept websocket client {peer_addr} failed: {e}");
            return;
        }
    };
    if let Err(auth_error) =
        check_auth_token(&mut ws_stream, &config.auth_tokens, Duration::from_secs(5)).await
    {
        println!("auth error [{peer_addr}]: {auth_error}");
        return;
    }
    //认证成功
    let ret_msg = Message::Binary(vec![0]);
    if let Err(ret_error) = ws_stream.send(ret_msg).await {
        println!("ret error: {ret_error}");
        return;
    }
    //println!("client auth success");
    if let Err(proxy_error) = run_proxy_tcp_loop(ws_stream).await {
        if !matches!(proxy_error, ProxyError::ClientClosed) {
            println!("{proxy_error}");
        }
    }
}
