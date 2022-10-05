use crate::common::msg::ServerMessage;
use actix_ws::{Closed, Session};
use bytes::Bytes;

///发送消息
pub async fn send_message<T: Into<ServerMessage>>(
    session: &mut Session,
    message: T,
) -> Result<(), Closed> {
    let server_msg = message.into();
    let msg_bytes: Bytes = server_msg.into();
    session.binary(msg_bytes).await
}
