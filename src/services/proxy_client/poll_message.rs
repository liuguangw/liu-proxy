use crate::common::msg::{ParseMessageError, ServerMessage};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use thiserror::Error;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

#[derive(Error, Debug)]
pub enum PollMessageError {
    #[error("protocol error, {0}")]
    Protocol(#[from] WsError),
    #[error("parse error, {0}")]
    Parse(#[from] ParseMessageError),
    #[error("server closed connection")]
    Closed,
}

///拉取二进制消息
pub async fn poll_message<T>(ws_stream: &mut T) -> Result<ServerMessage, PollMessageError>
where
    T: Stream<Item = Result<Message, WsError>> + Unpin,
{
    while let Some(message_result) = ws_stream.next().await {
        let message = message_result?;
        if let Message::Binary(data) = message {
            //解析消息
            let data_bytes = Bytes::from(data);
            let client_message = ServerMessage::try_from(data_bytes)?;
            return Ok(client_message);
        }
    }
    Err(PollMessageError::Closed)
}

impl PollMessageError {
    pub fn is_ws_error(&self) -> bool {
        matches!(self, Self::Closed | Self::Protocol(_))
    }
}
