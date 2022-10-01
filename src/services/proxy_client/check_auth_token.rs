use crate::services::poll_message;
use futures_util::SinkExt;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Error, Debug)]
pub enum ProxyAuthError {
    #[error("send auth data failed: {0}")]
    Send(WsError),
    #[error("poll auth data failed: {0}")]
    Poll(WsError),
    #[error("auth invalid")]
    AuthFailed,
}

pub async fn check_auth_token(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<(), ProxyAuthError> {
    let auth_token_msg = Message::text("123456");
    ws_stream
        .send(auth_token_msg)
        .await
        .map_err(ProxyAuthError::Send)?;
    match poll_message::poll_binary_message(ws_stream).await {
        Ok(option_s) => match option_s {
            Some(s) => {
                if let Some(0) = s.first() {
                    Ok(())
                } else {
                    Err(ProxyAuthError::AuthFailed)
                }
            }
            None => Err(ProxyAuthError::AuthFailed),
        },
        Err(e) => Err(ProxyAuthError::Poll(e)),
    }
}
