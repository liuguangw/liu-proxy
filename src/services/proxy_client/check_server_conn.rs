use crate::common::socket5::ConnDest;
use crate::services::poll_message;
use futures_util::{SinkExt, StreamExt};
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::Result as WsResult;

use super::io::ProxyConnectResult;

#[derive(Error, Debug)]
pub enum ProxyConnError {
    #[error("send auth data failed: {0}")]
    Send(WsError),
    #[error("poll auth data failed: {0}")]
    Poll(WsError),
    ///连接出错
    #[error("server connect failed: {0}")]
    Conn(String),
    ///超时
    #[error("server connect remote timeout")]
    Timeout,
}

impl ProxyConnError {
    ///判断客户端与服务端之间的连接是否有错误
    pub fn is_ws_error(&self) -> bool {
        matches!(self, Self::Send(_) | Self::Poll(_))
    }
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
        Ok(option_s) => match option_s {
            Some(s) => {
                let msg_type = s[0];
                if msg_type != 1 {
                    let error_message = format!("invalid msg_type: {msg_type}");
                    return Err(ProxyConnError::Conn(error_message));
                }
                let conn_result = ProxyConnectResult::from(&s[1..]);
                match conn_result {
                    ProxyConnectResult::Ok => Ok(()),
                    ProxyConnectResult::Err(e) => Err(ProxyConnError::Conn(e)),
                    ProxyConnectResult::Timeout => Err(ProxyConnError::Timeout),
                }
            }
            None => {
                let error_message = "poll conn result empty".to_string();
                Err(ProxyConnError::Conn(error_message))
            }
        },
        Err(e) => Err(ProxyConnError::Poll(e)),
    }
}
