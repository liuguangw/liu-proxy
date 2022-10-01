use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::Result as WsResult;
use tokio_tungstenite::tungstenite::Message;

pub async fn poll_text_message<T>(ws_stream: &mut T) -> WsResult<String>
where
    T: StreamExt<Item = WsResult<Message>> + Unpin,
{
    loop {
        if let Some(msg_result) = ws_stream.next().await {
            let msg = msg_result?;
            if let Message::Text(auth_token) = msg {
                return Ok(auth_token);
            }
        }
    }
}

pub async fn poll_binary_message<T>(ws_stream: &mut T) -> WsResult<Vec<u8>> where
T: StreamExt<Item = WsResult<Message>> + Unpin, {
    loop {
        if let Some(msg_result) = ws_stream.next().await {
            let msg = msg_result?;
            if let Message::Binary(data) = msg {
                return Ok(data);
            }
        }
    }
}
