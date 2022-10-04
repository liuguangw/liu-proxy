use super::proxy_error::ProxyError;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use actix_ws::{MessageStream, Session};

///处理连接逻辑
pub async fn handle_connection(session: Session, msg_stream: MessageStream) {
    if let Err(proxy_error) = run_proxy_tcp_loop(session, msg_stream).await {
        if !matches!(proxy_error, ProxyError::ClientClosed) {
            log::error!("{proxy_error}");
        }
    }
}
