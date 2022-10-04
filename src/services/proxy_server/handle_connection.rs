use super::check_auth_token::check_auth_token;
use super::proxy_error::ProxyError;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use crate::common::ServerConfig;
use actix_web::web;
use actix_ws::{MessageStream, Session};
use bytes::Bytes;
use std::time::Duration;

///处理连接逻辑
pub async fn handle_connection(
    mut session: Session,
    mut msg_stream: MessageStream,
    config: web::Data<ServerConfig>,
) {
    if let Err(auth_error) =
        check_auth_token(&mut msg_stream, &config.auth_tokens, Duration::from_secs(5)).await
    {
        log::error!("auth error: {auth_error}");
        _ = session.close(None).await;
        return;
    }
    //认证成功
    let ret_msg = Bytes::from_static(&[0]);
    if session.binary(ret_msg).await.is_err() {
        log::error!("ret error, client closed");
        return;
    }
    //println!("client auth success");
    if let Err(proxy_error) = run_proxy_tcp_loop(session, msg_stream).await {
        if !matches!(proxy_error, ProxyError::ClientClosed) {
            log::error!("{proxy_error}");
        }
    }
}
