use super::check_auth_token::check_auth_token;
use super::proxy_error::ProxyError;
use super::proxy_handshake::proxy_handshake;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use crate::common::NoServerCertVerifier;
use futures_util::{SinkExt, StreamExt};
use rustls::ClientConfig;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{
        client, error::UrlError, handshake::server::Response, http::Uri, stream::Mode,
        Error as WsError, Result as WsResult,
    },
    Connector, MaybeTlsStream, WebSocketStream,
};

pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    server_url: String,
    server_host: Option<String>,
    insecure: bool,
) {
    println!("client {addr} connected");
    let conn_dest = match proxy_handshake(&mut stream).await {
        Ok(s) => s,
        Err(handshake_error) => {
            println!("socket5 handshake failed: {handshake_error}");
            return;
        }
    };
    let mut ws_stream = match conn_websocket_server(&server_url, &server_host, insecure).await {
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
    insecure: bool,
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
    let connector = if insecure {
        //如果开启了不验证服务端ssl证书
        let mode = client::uri_mode(&request_uri)?;
        match mode {
            Mode::Plain => Some(Connector::Plain),
            Mode::Tls => {
                let client_config = ClientConfig::builder()
                    .with_safe_defaults()
                    .with_custom_certificate_verifier(Arc::new(NoServerCertVerifier {}))
                    .with_no_client_auth();
                let client_config = Arc::new(client_config);
                Some(Connector::Rustls(client_config))
            }
        }
    } else {
        None
    };
    tokio_tungstenite::client_async_tls_with_config(server_url, stream, None, connector).await
}
