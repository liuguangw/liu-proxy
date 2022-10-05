use crate::common::msg::ClientMessage;
use bytes::Bytes;
use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

///发送消息
pub async fn send_message<S, T>(sender: &mut S, message: T) -> Result<(), WsError>
where
    S: Sink<Message, Error = WsError> + Unpin,
    T: Into<ClientMessage>,
{
    let server_msg = message.into();
    let msg_bytes: Bytes = server_msg.into();
    sender.send(Message::Binary(msg_bytes.to_vec())).await
}
