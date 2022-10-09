use crate::common::msg::{ClientMessage, ParseMessageError};
use actix_ws::{Message, ProtocolError, Session};
use futures_util::StreamExt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PollMessageError {
    #[error("protocol error, {0}")]
    Protocol(#[from] ProtocolError),
    #[error("parse error, {0}")]
    Parse(#[from] ParseMessageError),
    #[error("client closed connection")]
    Closed,
}

///拉取二进制消息
pub async fn poll_message<T>(
    session: &mut Session,
    ws_stream: &mut T,
) -> Result<ClientMessage, PollMessageError>
where
    T: StreamExt<Item = Result<Message, ProtocolError>> + Unpin,
{
    while let Some(message_result) = ws_stream.next().await {
        let message = message_result?;
        if let Message::Binary(data) = message {
            //解析消息
            let client_message = ClientMessage::try_from(data)?;
            return Ok(client_message);
        } else if let Message::Ping(bytes) = message {
            //回复ping消息
            if session.pong(&bytes).await.is_err() {
                return Err(PollMessageError::Closed);
            }
        } else if let Message::Close(option_reason) = message {
            //回复close
            let session = session.clone();
            _ = session.close(option_reason).await;
            return Err(PollMessageError::Closed);
        }
    }
    Err(PollMessageError::Closed)
}
