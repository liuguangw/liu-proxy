use super::check_auth::CheckAuth;
use super::handle_connection;
use axum::response::IntoResponse;
use axum::{
    extract::WebSocketUpgrade,
    response::{AppendHeaders, Response},
};
use http::{header::CONTENT_TYPE, StatusCode};
use tokio::fs;

///处理websocket连接
pub async fn ws_handler(
    auth_data: Option<CheckAuth>,
    ws_opt: Option<WebSocketUpgrade>,
) -> Response {
    //身份认证
    let auth_data = match auth_data {
        Some(s) => s,
        None => return ws_error_handler().await,
    };
    //执行websocket协议握手
    match ws_opt {
        Some(ws) => ws.on_upgrade(|ws_stream| {
            handle_connection::handle_connection(ws_stream, auth_data.user)
        }),
        None => ws_error_handler().await,
    }
}

///认证失败、握手失败时显示404错误页面
async fn ws_error_handler() -> Response {
    let error_file_data = match fs::read("./web/404.html").await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    (
        StatusCode::NOT_FOUND,
        AppendHeaders([(CONTENT_TYPE, "text/html; charset=utf-8")]),
        error_file_data,
    )
        .into_response()
}
