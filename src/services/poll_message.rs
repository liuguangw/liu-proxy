use futures_util::StreamExt;
use futures_util::TryStreamExt;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::Result as WsResult;

pub async fn poll_text_message<T>(ws_stream: &mut T) -> WsResult<Option<String>>
where
    T: StreamExt<Item = WsResult<Message>> + Unpin,
{
    loop {
        let option_msg = ws_stream.try_next().await?;
        match option_msg {
            Some(msg) => {
                if let Message::Text(auth_token) = msg {
                    return Ok(Some(auth_token));
                }
            }
            None => return Ok(None),
        };
    }
}

pub async fn poll_binary_message<T>(ws_stream: &mut T) -> WsResult<Option<Vec<u8>>>
where
    T: StreamExt<Item = WsResult<Message>> + Unpin,
{
    loop {
        let option_msg = ws_stream.try_next().await?;
        match option_msg {
            Some(msg) => {
                if let Message::Binary(data) = msg {
                    return Ok(Some(data));
                }
            }
            None => return Ok(None),
        };
    }
}
