use super::{connection::ConnectionError, http::parse_request::ParseRequestError};
use std::io::Error as IoError;
use thiserror::Error;

///客户端proxy错误定义
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("read request failed: {0}")]
    ReadRequest(IoError),
    #[error("write response failed: {0}")]
    WriteResponse(IoError),
    #[error("{0}")]
    ConnIoErr(#[from] ConnectionError),
    #[error("parse request failed: {0}")]
    ParseRequest(#[from] httparse::Error),
    #[error("parse request length failed: {0}")]
    ParseLength(#[from] ParseRequestError),
}
impl ProxyError {
    pub fn is_ws_error(&self) -> bool {
        match self {
            Self::ConnIoErr(e) => e.is_ws_error(),
            _ => false,
        }
    }
}
