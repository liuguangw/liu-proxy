use actix_ws::{Message, ProtocolError, Session};
use bytes::Bytes;
use futures_util::StreamExt;

///拉取二进制消息
pub async fn poll_binary_message<T>(
    session: &mut Session,
    ws_stream: &mut T,
) -> Option<Result<Bytes, ProtocolError>>
where
    T: StreamExt<Item = Result<Message, ProtocolError>> + Unpin,
{
    while let Some(message_result) = ws_stream.next().await {
        let message = match message_result {
            Ok(s) => s,
            Err(e) => return Some(Err(e)),
        };
        if let Message::Binary(data) = message {
            return Some(Ok(data));
        } else if let Message::Ping(bytes) = message {
            if session.pong(&bytes).await.is_err() {
                return None;
            }
        }
    }
    None
}
