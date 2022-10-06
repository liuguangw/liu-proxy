use super::ConfigError;
use std::io::Error as IoError;
use thiserror::Error;

///启动客户端的错误
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("load {0} failed: {1}")]
    Config(String, ConfigError),
    #[error("bind address {0} failed: {1}")]
    Bind(String, IoError),
    #[error("wait signal failed: {0}")]
    WaitSignal(IoError),
}
