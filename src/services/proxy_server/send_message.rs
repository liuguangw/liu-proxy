use crate::common::msg::ServerMessage;
use axum::extract::ws::Message;
use axum::Error as WsError;
use bytes::Bytes;
use futures_util::{Sink, SinkExt};

///发送消息
pub async fn send_message<T>(ws_stream: &mut T, server_msg: ServerMessage) -> Result<(), WsError>
where
    T: Sink<Message, Error = WsError> + Unpin,
{
    let msg_bytes: Bytes = server_msg.into();
    let message = Message::Binary(msg_bytes.to_vec());
    ws_stream.send(message).await
}
