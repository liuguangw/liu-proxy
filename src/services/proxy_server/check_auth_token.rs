use crate::services::poll_message;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::Error as WsErr;
use tokio_tungstenite::WebSocketStream;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Error during poll auth_token, {0}")]
    WsErr(#[from] WsErr),
    #[error("client closed connection")]
    ClientClosed,
    #[error("wait auth_token timeout")]
    Timeout,
    #[error("invalid auth token")]
    InvalidToken,
}

pub async fn check_auth_token(ws_stream: &mut WebSocketStream<TcpStream>) -> Result<(), AuthError> {
    let timeout_duration = Duration::from_secs(5);
    let auth_token =
        match timeout(timeout_duration, poll_message::poll_text_message(ws_stream)).await {
            Ok(auth_token_result) => match auth_token_result? {
                Some(s) => s,
                None => return Err(AuthError::ClientClosed),
            },
            Err(_) => return Err(AuthError::Timeout),
        };
    if auth_token != "123456" {
        return Err(AuthError::InvalidToken);
    }
    Ok(())
}
