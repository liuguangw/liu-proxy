use super::poll_message;
use super::poll_message::PollMessageError;
use crate::common::{
    msg::client::Connect, msg::server::ConnectResult, msg::ServerMessage, socket5::ConnDest,
};
use futures_util::{Sink, Stream};
use thiserror::Error;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

#[derive(Error, Debug)]
pub enum ConnectError {
    #[error("send conn msg failed: {0}")]
    Send(#[from] WsError),
    #[error("poll conn message failed: {0}")]
    PollMessage(#[from] PollMessageError),
    #[error("not connection message")]
    NotConnMessage,
    //服务端返回的连接失败信息
    #[error("{0}")]
    ConnErr(String),
    ///超时
    #[error("server connect remote timeout")]
    Timeout,
}

pub async fn check_server_conn<T>(
    ws_stream: &mut T,
    conn_dest: &ConnDest,
) -> Result<(), ConnectError>
where
    T: Stream<Item = Result<Message, WsError>> + Sink<Message, Error = WsError> + Unpin,
{
    let conn_msg = Connect(conn_dest.to_string());
    super::send_message::send_message(ws_stream, conn_msg).await?;
    let message = poll_message::poll_message(ws_stream).await?;
    match message {
        ServerMessage::ConnResult(conn_result) => match conn_result {
            ConnectResult::Ok => Ok(()),
            ConnectResult::Err(e) => Err(ConnectError::ConnErr(e)),
            ConnectResult::Timeout => Err(ConnectError::Timeout),
        },
        _ => Err(ConnectError::NotConnMessage),
    }
}

impl ConnectError {
    pub fn is_ws_error(&self) -> bool {
        match self {
            ConnectError::Send(_) => true,
            ConnectError::PollMessage(e) => e.is_ws_error(),
            _ => false,
        }
    }
}
