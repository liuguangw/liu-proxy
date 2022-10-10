use super::proxy_error::ProxyError;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use axum::extract::ws::WebSocket;

///处理连接逻辑
pub async fn handle_connection(mut ws_stream: WebSocket, username: String) {
    log::info!("user {username} connected");
    if let Err(proxy_error) = run_proxy_tcp_loop(&mut ws_stream).await {
        match proxy_error {
            ProxyError::ClientClosed => (),
            ProxyError::NotConnMessage | ProxyError::NotRequestMessage => {
                //关闭连接
                _ = ws_stream.close().await;
                log::error!("{proxy_error}");
            }
            _ => {
                log::error!("{proxy_error}");
            }
        };
    }
    log::info!("user {username} disconnected");
}
