use crate::common::{ClientConfig, NoServerCertVerifier, WebsocketRequest};
use rustls::ClientConfig as TlsClientConfig;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{error::UrlError, handshake::server::Response, http::uri::Uri, Error as WsError},
    Connector, MaybeTlsStream, WebSocketStream,
};

///处理客户端与服务端之间的握手操作
pub async fn auth_handshake(
    config: &ClientConfig,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), WsError> {
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
    let ws_request = WebsocketRequest::new(&config.server_url, &config.auth_token);
    //常规模式
    if config.server_host.is_empty() {
        //根据url进行连接
        return tokio_tungstenite::connect_async_tls_with_config(ws_request, None, connector).await;
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
    tokio_tungstenite::client_async_tls_with_config(ws_request, stream, None, connector).await
}
