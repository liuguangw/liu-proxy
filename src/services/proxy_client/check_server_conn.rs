use std::str::Utf8Error;

use crate::common::socket5::ConnDest;
use crate::services::poll_message;
use futures_util::{SinkExt, StreamExt};
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::Result as WsResult;

#[derive(Error, Debug)]
pub enum ProxyConnError {
    #[error("send auth data failed: {0}")]
    Send(WsError),
    #[error("poll auth data failed: {0}")]
    Poll(WsError),
    ///连接出错
    #[error("server conn failed: {0}")]
    Conn(String),
    #[error("parse server error msg as utf-8 failed: {0}")]
    Utf8Message(#[from] Utf8Error)
}

pub async fn check_server_conn<T>(
    ws_stream: &mut T,
    conn_dest: &ConnDest,
) -> Result<(), ProxyConnError>
where
    T: StreamExt<Item = WsResult<Message>> + SinkExt<Message, Error = WsError> + Unpin,
{
    let conn_msg = Message::binary(conn_dest.to_raw_data());
    if let Err(e) = ws_stream.send(conn_msg).await {
        return Err(ProxyConnError::Send(e));
    }
    match poll_message::poll_binary_message(ws_stream).await {
        Ok(s) => match s.first() {
            Some(0)=>{
                Ok(())
            },
            Some(1)|Some(2)=>{
                let error_message =std::str::from_utf8(&s[1..])?.to_string();
                Err(ProxyConnError::Conn(error_message))
            },
            _=>{
                let error_message ="invalid conn ret code".to_string();
                Err(ProxyConnError::Conn(error_message))
            }
        },
        Err(e) => Err(ProxyConnError::Poll(e)),
    }
}
