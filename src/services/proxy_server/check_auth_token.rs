use crate::services::poll_message;
use futures_util::StreamExt;
use std::time::Duration;
use thiserror::Error;
use tokio::time;
use tokio_tungstenite::tungstenite::Error as WsErr;
use tokio_tungstenite::tungstenite::{Message, Result as WsResult};

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

pub async fn check_auth_token<T>(
    ws_stream: &mut T,
    auth_tokens: &[String],
    timeout_duration: Duration,
) -> Result<(), AuthError>
where
    T: StreamExt<Item = WsResult<Message>> + Unpin,
{
    let auth_token =
        match time::timeout(timeout_duration, poll_message::poll_text_message(ws_stream)).await {
            Ok(auth_token_result) => match auth_token_result? {
                Some(s) => s,
                None => return Err(AuthError::ClientClosed),
            },
            Err(_) => return Err(AuthError::Timeout),
        };
    if !auth_tokens.contains(&auth_token) {
        return Err(AuthError::InvalidToken);
    }
    Ok(())
}
