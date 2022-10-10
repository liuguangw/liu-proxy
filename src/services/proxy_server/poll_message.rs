use crate::common::msg::{ClientMessage, ParseMessageError};
use axum::extract::ws::Message;
use axum::Error as WsError;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PollMessageError {
    #[error("protocol error, {0}")]
    Protocol(#[from] WsError),
    #[error("parse error, {0}")]
    Parse(#[from] ParseMessageError),
    #[error("client closed connection")]
    Closed,
}

///拉取二进制消息
pub async fn poll_message<T>(ws_stream: &mut T) -> Result<ClientMessage, PollMessageError>
where
    T: Stream<Item = Result<Message, WsError>> + Unpin,
{
    //log::info!("before loop");
    while let Some(message_result) = ws_stream.next().await {
        //log::info!("enter loop1");
        let message = message_result?;
        //log::info!("enter loop2");
        if let Message::Binary(data) = message {
            //解析消息
            let data_bytes = Bytes::from(data);
            let client_message = ClientMessage::try_from(data_bytes)?;
            return Ok(client_message);
        }
    }
    //log::info!("enter loop3");
    Err(PollMessageError::Closed)
}
