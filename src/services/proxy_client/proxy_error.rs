use super::{http::parse_request::ParseRequestError, poll_message::PollMessageError};
use std::io::Error as IoError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsError;

///客户端proxy错误定义
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("read request failed: {0}")]
    ReadRequest(IoError),
    #[error("write response failed: {0}")]
    WriteResponse(IoError),
    #[error("send request failed: {0}")]
    SendRequest(WsError),
    #[error("read response failed: {0}")]
    PollMessage(PollMessageError),
    #[error("invalid server message")]
    InvalidServerMessage,
    #[error("server request failed: {0}")]
    ServerRequest(String),
    #[error("server fetch response failed: {0}")]
    ServerResponse(String),
    #[error("parse request failed: {0}")]
    ParseRequest(#[from] httparse::Error),
    #[error("parse request length failed: {0}")]
    ParseLength(#[from] ParseRequestError),
}
impl ProxyError {
    pub fn is_ws_error(&self) -> bool {
        match self {
            ProxyError::SendRequest(_) => true,
            ProxyError::PollMessage(e) => e.is_ws_error(),
            _ => false,
        }
    }
}
