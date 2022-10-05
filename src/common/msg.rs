///客户端产生的消息子类
pub mod client;
mod client_message;
///服务端产生的消息子类
pub mod server;
mod server_message;

pub use client_message::ClientMessage;
pub use server_message::ServerMessage;
use std::str::Utf8Error;
use thiserror::Error;

///解析消息的错误定义
#[derive(Error, Debug)]
pub enum ParseMessageError {
    ///消息长度不完整
    #[error("incomplete message")]
    Incomplete,
    #[error("invalid utf-8 string, {0}")]
    Utf8Err(#[from] Utf8Error),
    #[error("invalid response status {0}")]
    InvalidResponseStatus(u8),
    #[error("invalid conn status {0}")]
    InvalidConnStatus(u8),
    #[error("invalid message type {0}")]
    InvalidMsgType(u8),
}
