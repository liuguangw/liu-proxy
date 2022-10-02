use crate::services::poll_message;
use futures_util::SinkExt;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("send auth data failed: {0}")]
    Send(WsError),
    #[error("poll auth data failed: {0}")]
    Poll(WsError),
    #[error("auth invalid")]
    AuthFailed,
    #[error("connection closed by server")]
    ServerClosed,
}

pub async fn check_auth_token(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<(), AuthError> {
    let auth_token_msg = Message::text("123456");
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
