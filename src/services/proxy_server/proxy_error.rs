use crate::common::socket5::ParseConnDestError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio_tungstenite::tungstenite::{Error as WsErr, Message};

///服务端proxy错误定义
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("parse client conn dest failed: {0}")]
    ParseClientDest(#[from] ParseConnDestError),
    #[error("{0} failed: {1}")]
    WsErr(String, WsErr),
    #[error("client closed connection")]
    ClientClosed,
    #[error("channel send failed: {0}")]
    Channel(#[from] SendError<Message>),
}

impl ProxyError {
    pub fn ws_err(tip: &str, e: WsErr) -> Self {
        Self::WsErr(tip.to_string(), e)
    }
}
