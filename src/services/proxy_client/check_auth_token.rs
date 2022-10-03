use std::time::Duration;

use crate::services::poll_message;
use futures_util::SinkExt;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::time;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("send auth data failed, {0}")]
    Send(WsError),
    #[error("poll auth data failed, {0}")]
    Poll(WsError),
    #[error("auth ret code invalid")]
    AuthFailed,
    #[error("connection closed by server")]
    ServerClosed,
    #[error("auth timeout")]
    Timeout,
}

pub async fn check_auth_token(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    token: &str,
    timeout_duration: Duration,
) -> Result<(), AuthError> {
    match time::timeout(timeout_duration, check_auth(ws_stream, token)).await {
        Ok(check_result) => check_result,
        Err(_) => Err(AuthError::Timeout),
    }
}

async fn check_auth(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    token: &str,
) -> Result<(), AuthError> {
    let auth_token_msg = Message::text(token);
    if let Err(e) = ws_stream.send(auth_token_msg).await {
        return Err(AuthError::Send(e));
    }
    match poll_message::poll_binary_message(ws_stream).await {
        Ok(option_s) => match option_s {
            Some(s) => {
                if let Some(0) = s.first() {
                    Ok(())
                } else {
                    Err(AuthError::AuthFailed)
                }
            }
            None => Err(AuthError::ServerClosed),
        },
        Err(e) => Err(AuthError::Poll(e)),
    }
}
