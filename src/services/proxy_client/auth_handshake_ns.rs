use super::check_auth_token::{check_auth_token, AuthError};
use crate::common::{ClientConfig, NoServerCertVerifier};
use rustls::ClientConfig as TlsClientConfig;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{
        client, error::UrlError, handshake::server::Response, http::Uri, stream::Mode,
        Error as WsError, Result as WsResult,
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
    //常规模式
    if config.server_host.is_empty() && !config.insecure {
        //根据url进行连接
        return tokio_tungstenite::connect_async(&config.server_url).await;
    }
    let request_uri = config.server_url.parse::<Uri>()?;
    //获取连接地址: server_host优先,其次是url中的host
    let tcp_host = if config.server_host.is_empty() {
        match request_uri.host() {
            Some(d) => d,
            None => return Err(WsError::Url(UrlError::NoHostName)),
        }
    } else {
        &config.server_host
    };
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
    let addr = format!("{tcp_host}:{port}");
    let stream = TcpStream::connect(addr).await.map_err(WsError::Io)?;
    //tls connector
    let connector = if config.insecure {
        //跳过域名证书验证的配置
        let mode = client::uri_mode(&request_uri)?;
        match mode {
            Mode::Plain => Some(Connector::Plain),
            Mode::Tls => {
                let client_config = TlsClientConfig::builder()
                    .with_safe_defaults()
                    .with_custom_certificate_verifier(Arc::new(NoServerCertVerifier {}))
                    .with_no_client_auth();
                let client_config = Arc::new(client_config);
                Some(Connector::Rustls(client_config))
            }
        }
    } else {
        //默认配置
        None
    };
    tokio_tungstenite::client_async_tls_with_config(&config.server_url, stream, None, connector)
        .await
}
