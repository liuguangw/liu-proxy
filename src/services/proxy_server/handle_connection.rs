use std::time::SystemTime;

use super::client_session::ClientSession;
use axum::extract::ws::WebSocket;

///处理连接逻辑
pub async fn handle_connection(ws_stream: WebSocket, username: String) {
    log::info!("user {username} connected");
    //开始计时
    let time_start = SystemTime::now();
    //
    let mut client_session = ClientSession::new(username);
    if let Err(proxy_error) = client_session.run_proxy(ws_stream).await {
        log::error!("{proxy_error}");
    }
    //结束计时
    let time_end = SystemTime::now();
    let d = time_end.duration_since(time_start).unwrap();
    log::info!(
        "user {} disconnected, use_count={}, duration={}s",
        &client_session.username,
        client_session.use_count,
        d.as_secs()
    );
}
