use std::io::Error as IoErr;
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsErr;

///客户端proxy错误定义
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("{0} failed: {1}")]
    WsErr(String, WsErr),
    #[error("{0} failed: {1}")]
    IoErr(String, IoErr),
    ///客户端主动断开本地socket5服务器
    #[error("client closed")]
    ClientClosed,
    ///远端主动关闭了连接
    #[error("client closed")]
    RemoteClosed,
    ///
    #[error("server write request failed: {0}")]
    RequestErr(String),
    ///
    #[error("server read response failed: {0}")]
    ResponseErr(String),
}

impl ProxyError {
    pub fn ws_err(tip: &str, e: WsErr) -> Self {
        Self::WsErr(tip.to_string(), e)
    }
    pub fn io_err(tip: &str, e: IoErr) -> Self {
        Self::IoErr(tip.to_string(), e)
    }
}
