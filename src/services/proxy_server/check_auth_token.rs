use super::poll_message;
use actix_ws::{Message, ProtocolError};
use futures_util::StreamExt;
use std::time::Duration;
use thiserror::Error;
use tokio::time;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Error during poll auth_token, {0}")]
    WsErr(#[from] ProtocolError),
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
    T: StreamExt<Item = Result<Message, ProtocolError>> + Unpin,
{
    let auth_token =
        match time::timeout(timeout_duration, poll_message::poll_text_message(ws_stream)).await {
            Ok(option_token_result) => match option_token_result {
                Some(s) => s?,
                None => return Err(AuthError::ClientClosed),
            },
            Err(_) => return Err(AuthError::Timeout),
        };
    if !auth_tokens.contains(&auth_token) {
        return Err(AuthError::InvalidToken);
    }
    Ok(())
}
