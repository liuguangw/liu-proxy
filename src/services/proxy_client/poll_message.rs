use futures_util::{StreamExt, TryStreamExt};
use tokio_tungstenite::tungstenite::{Message, Result as WsResult};

///拉取二进制消息
pub async fn poll_binary_message<T>(ws_stream: &mut T) -> WsResult<Option<Vec<u8>>>
where
    T: StreamExt<Item = WsResult<Message>> + Unpin,
{
    while let Some(msg) = ws_stream.try_next().await? {
        if let Message::Binary(data) = msg {
            return Ok(Some(data));
        }
    }
    Ok(None)
}
