use super::super::check_server_conn::ConnectError as ServerConnectError;
use crate::common::msg::ParseMessageError;
use std::io::Error as IoError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsError;

///客户端proxy错误定义
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("dest blocked by route")]
    RouteBlocked,
    #[error("websocket conn failed, {0}")]
    WsConn(WsError),
    #[error("{0}")]
    ServerConn(ServerConnectError),
    #[error("{0}")]
    TcpConn(IoError),
    #[error("tcp write failed, {0}")]
    TcpWrite(IoError),
    #[error("ws write failed, {0}")]
    WsWrite(WsError),
    #[error("tcp read failed, {0}")]
    TcpRead(IoError),
    #[error("ws read failed, {0}")]
    WsRead(WsError),
    #[error("parse server message failed, {0}")]
    WsParseMsg(ParseMessageError),
    ///与服务端之间的连接或者直连连接被断开
    #[error("remote connection closed")]
    ConnClosed,
    ///remote断开了服务端
    #[error("ws server conn closed")]
    WsRemoteClosed,
    ///server发出请求失败
    #[error("{0}")]
    WsServerRequest(String),
    ///server读取response失败
    #[error("{0}")]
    WsServerResponse(String),
    #[error("invalid server message")]
    WsInvalidServerMessage,
}

impl ConnectionError {
    pub fn is_ws_error(&self) -> bool {
        matches!(self, Self::WsConn(_) | Self::WsRead(_) | Self::WsWrite(_))
    }
}
