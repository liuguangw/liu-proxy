use super::poll_message::PollMessageError;
use axum::Error as WsError;
use thiserror::Error;

///服务端proxy错误定义
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("poll message failed: {0}")]
    PollMessage(PollMessageError),
    #[error("send message failed: {0}")]
    SendMessage(WsError),
    #[error("channel send message failed")]
    ChannelSendMessage,
    #[error("client closed connection")]
    ClientClosed,
    #[error("not connection message")]
    NotConnMessage,
    #[error("not request message")]
    NotRequestMessage,
}

impl From<PollMessageError> for ProxyError {
    fn from(item: PollMessageError) -> Self {
        match item {
            PollMessageError::Closed => Self::ClientClosed,
            _ => Self::PollMessage(item),
        }
    }
}
