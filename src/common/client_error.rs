use super::{ConfigError, ParseWebsocketRequestError};
use crate::services::geosite::{FromBinaryError, ParseSelectionError};
use std::io::Error as IoError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsError;

///启动客户端的错误
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("load {0} failed: {1}")]
    Config(String, ConfigError),
    #[error("bind address {0} failed: {1}")]
    Bind(String, IoError),
    #[error("wait signal failed: {0}")]
    WaitSignal(IoError),
    #[error("init manger failed: {0}")]
    InitManger(#[from] ParseWebsocketRequestError),
    #[error("check server status failed: {0}")]
    CheckConn(WsError),
    #[error("run http service failed: {0}")]
    HttpService(IoError),
    #[error("load geosite data failed: {0}")]
    LoadGeoSite(#[from] FromBinaryError),
    #[error("parse route selection failed: {0}")]
    ParseRouteSelection(#[from] ParseSelectionError),
}
