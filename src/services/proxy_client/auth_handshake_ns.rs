use super::check_auth_token::{check_auth_token, AuthError};
use crate::common::{ClientConfig, NoServerCertVerifier};
use rustls::ClientConfig as TlsClientConfig;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{
        error::UrlError, handshake::server::Response, http::Uri, Error as WsError,
        Result as WsResult,
    },
    Connector, MaybeTlsStream, WebSocketStream,
};

#[derive(Error, Debug)]
pub enum AuthHandshakeError {
    #[error("websocket handshake failed: {0}")]
    WsError(#[from] WsError),
    #[error("auth failed: {0}")]
    Auth(#[from] AuthError),
}

///处理客户端与服务端之间的握手操作
pub async fn auth_handshake(
    config: &ClientConfig,
    timeout_duration: Duration,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), AuthHandshakeError> {
    let (mut ws_stream, response) = conn_websocket_server(config).await?;
    check_auth_token(&mut ws_stream, &config.auth_token, timeout_duration).await?;
    Ok((ws_stream, response))
}
///根据客户端配置连接到服务端
async fn conn_websocket_server(
    config: &ClientConfig,
) -> WsResult<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response)> {
    //tls connector
    let connector = if config.insecure {
        //跳过ssl证书验证
        let client_config = TlsClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(NoServerCertVerifier {}))
            .with_no_client_auth();
        let client_config = Arc::new(client_config);
        Some(Connector::Rustls(client_config))
    } else {
        //默认配置
        None
    };
    //常规模式
    if config.server_host.is_empty() {
        //根据url进行连接
        return tokio_tungstenite::connect_async_tls_with_config(
            &config.server_url,
            None,
            connector,
        )
        .await;
    }
    //连接到config.server_host:port
    let request_uri = config.server_url.parse::<Uri>()?;
    //port
    let port = request_uri
        .port_u16()
        .or_else(|| match request_uri.scheme_str() {
            Some("wss") => Some(443),
            Some("ws") => Some(80),
            _ => None,
        })
        .ok_or(WsError::Url(UrlError::UnsupportedUrlScheme))?;
    //建立tcp连接
    let addr = format!("{}:{port}", config.server_host);
    let stream = TcpStream::connect(addr).await.map_err(WsError::Io)?;
    tokio_tungstenite::client_async_tls_with_config(&config.server_url, stream, None, connector)
        .await
}
