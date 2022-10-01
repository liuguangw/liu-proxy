use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsErr;
use std::io::Error as IoErr;

///客户端proxy错误定义
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("{0} failed: {1}")]
    WsErr(String, WsErr),
    #[error("{0} failed: {1}")]
    IoErr(String, IoErr),
}

impl ProxyError {
    pub fn ws_err(tip: &str, e: WsErr) -> Self {
        Self::WsErr(tip.to_string(), e)
    }
    pub fn io_err(tip: &str, e: IoErr) -> Self {
        Self::IoErr(tip.to_string(), e)
    }
}
