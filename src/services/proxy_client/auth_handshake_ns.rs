use crate::common::{NoServerCertVerifier, WebsocketRequest};
use rustls::ClientConfig as TlsClientConfig;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{handshake::server::Response, Error as WsError},
    Connector, MaybeTlsStream, WebSocketStream,
};

///处理客户端与服务端之间的握手操作
pub async fn auth_handshake(
    ws_request: &WebsocketRequest,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), WsError> {
    //tls connector
    let connector = if ws_request.insecure {
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
    //建立tcp连接
    //log::info!("tcp conn: {}", ws_request.server_addr);
    let stream = TcpStream::connect(ws_request.server_addr)
        .await
        .map_err(WsError::Io)?;
    //websocket握手
    tokio_tungstenite::client_async_tls_with_config(ws_request, stream, None, connector).await
}
