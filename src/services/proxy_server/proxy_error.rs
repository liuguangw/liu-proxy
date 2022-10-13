use crate::common::msg::ParseMessageError;
use axum::Error as WsError;
use thiserror::Error;

///服务端proxy错误定义
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("read client failed: {0}")]
    ReadClient(WsError),
    #[error("write client failed: {0}")]
    WriteClient(WsError),
    #[error("reader channel closed")]
    ReadChannel,
    #[error("write channel closed")]
    WriteChannel,
    #[error("parse message failed: {0}")]
    ParseMessage(#[from] ParseMessageError),
}
