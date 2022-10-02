use super::check_auth_token::check_auth_token;
use super::proxy_error::ProxyError;
use super::proxy_handshake::proxy_handshake;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{
        error::UrlError, handshake::server::Response, http::Uri, Error as WsError,
        Result as WsResult,
    },
    MaybeTlsStream, WebSocketStream,
};

pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    server_url: String,
    server_host: Option<String>,
) {
    println!("client {addr} connected");
    let conn_dest = match proxy_handshake(&mut stream).await {
        Ok(s) => s,
        Err(handshake_error) => {
            println!("socket5 handshake failed: {handshake_error}");
            return;
        }
    };
    let mut ws_stream = match conn_websocket_server(&server_url, &server_host).await {
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
    let (mut ws_writer, mut ws_reader) = ws_stream.split();
    if let Err(proxy_error) =
        run_proxy_tcp_loop(&mut ws_reader, &mut ws_writer, &mut stream, &conn_dest).await
    {
        if !matches!(proxy_error, ProxyError::ClientClosed) {
            println!("{proxy_error}");
        }
        //断开与server之间的连接
        if !matches!(
            proxy_error,
            ProxyError::WsErr(_, _) | ProxyError::ServerClosed
        ) {
            if let Err(e1) = ws_writer.close().await {
                println!("close conn failed: {e1}");
            }
        }
    }
}

async fn conn_websocket_server(
    server_url: &str,
    server_host: &Option<String>,
) -> WsResult<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response)> {
    let host = match server_host {
        Some(s) => s,
        None => return tokio_tungstenite::connect_async(server_url).await,
    };
    let request_uri = server_url.parse::<Uri>()?;
    let port = request_uri
        .port_u16()
        .or_else(|| match request_uri.scheme_str() {
            Some("wss") => Some(443),
            Some("ws") => Some(80),
            _ => None,
        })
        .ok_or(WsError::Url(UrlError::UnsupportedUrlScheme))?;
    let addr = format!("{host}:{port}");
    let stream = TcpStream::connect(addr).await.map_err(WsError::Io)?;
    tokio_tungstenite::client_async_tls_with_config(server_url, stream, None, None).await
}
