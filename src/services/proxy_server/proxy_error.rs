use crate::common::socket5::ParseConnDestError;
use actix_ws::ProtocolError;
use thiserror::Error;

///服务端proxy错误定义
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("parse client conn dest failed: {0}")]
    ParseClientDest(#[from] ParseConnDestError),
    #[error("{0} failed: {1}")]
    WsErr(String, ProtocolError),
    #[error("client closed connection")]
    ClientClosed,
}

impl ProxyError {
    pub fn ws_err(tip: &str, e: ProtocolError) -> Self {
        Self::WsErr(tip.to_string(), e)
    }
}
