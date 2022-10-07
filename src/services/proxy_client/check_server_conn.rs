use super::poll_message::PollMessageError;
use super::{poll_message, server_conn_manger::ConnPair};
use crate::common::{
    msg::client::Connect, msg::server::ConnectResult, msg::ServerMessage, socks5::ConnDest,
};
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsError;

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

pub async fn check_server_conn(
    ws_conn_pair: &mut ConnPair,
    conn_dest: &ConnDest,
) -> Result<(), ConnectError> {
    let conn_msg = Connect(conn_dest.to_string());
    super::send_message::send_message(&mut ws_conn_pair.0, conn_msg).await?;
    let message = poll_message::poll_message(&mut ws_conn_pair.1).await?;
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
