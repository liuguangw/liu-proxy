use super::io::ProxyConnectResult;
use super::poll_message;
use crate::common::socket5::ConnDest;
use futures_util::{SinkExt, StreamExt};
use thiserror::Error;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

#[derive(Error, Debug)]
pub enum ProxyConnError {
    #[error("send auth data failed: {0}")]
    Send(WsError),
    #[error("poll auth data failed: {0}")]
    Poll(WsError),
    ///连接出错
    #[error("{0}")]
    Conn(String),
    ///超时
    #[error("server connect remote timeout")]
    Timeout,
    #[error("connection closed by server")]
    ServerClosed,
}

impl ProxyConnError {
    ///判断客户端与服务端之间的连接是否有错误
    pub fn is_ws_error(&self) -> bool {
        matches!(self, Self::Send(_) | Self::Poll(_) | Self::ServerClosed)
    }
}

pub async fn check_server_conn<T>(
    ws_stream: &mut T,
    conn_dest: &ConnDest,
) -> Result<(), ProxyConnError>
where
    T: StreamExt<Item = Result<Message, WsError>> + SinkExt<Message, Error = WsError> + Unpin,
{
    let conn_msg = Message::binary(conn_dest.to_raw_data());
    if let Err(e) = ws_stream.send(conn_msg).await {
        return Err(ProxyConnError::Send(e));
    }
    match poll_message::poll_binary_message(ws_stream).await {
        Ok(option_s) => match option_s {
            Some(binary_data) => parse_server_conn_result(&binary_data),
            None => Err(ProxyConnError::ServerClosed),
        },
        Err(e) => Err(ProxyConnError::Poll(e)),
    }
}

fn parse_server_conn_result(binary_data: &[u8]) -> Result<(), ProxyConnError> {
    let msg_type = binary_data[0];
    if msg_type != 1 {
        let error_message = format!("invalid msg_type: {msg_type}");
        return Err(ProxyConnError::Conn(error_message));
    }
    let conn_result = ProxyConnectResult::from(&binary_data[1..]);
    match conn_result {
        ProxyConnectResult::Ok => Ok(()),
        ProxyConnectResult::Err(e) => Err(ProxyConnError::Conn(e)),
        ProxyConnectResult::Timeout => Err(ProxyConnError::Timeout),
    }
}
