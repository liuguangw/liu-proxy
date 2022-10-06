use super::ConfigError;
use rustls::Error as TlsError;
use std::io::Error as IoError;
use thiserror::Error;

///启动服务端的错误
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("load {0} failed: {1}")]
    Config(String, ConfigError),
    #[error("ssl cert path not set")]
    ConfigSSlCertNone,
    #[error("ssl key path not set")]
    ConfigSSlKeyNone,
    #[error("load ssl cert failed: {0}")]
    Cert(#[from] TlsServerConfigError),
    #[error("bind address {0} failed: {1}")]
    Bind(String, IoError),
    #[error("run http service failed: {0}")]
    HttpService(IoError),
}

///服务端ssl证书加载错误
#[derive(Error, Debug)]
pub enum TlsServerConfigError {
    #[error("load public cert failed, {0}")]
    LoadCert(IoError),
    #[error("parse public cert failed, {0}")]
    ParseCert(IoError),
    #[error("load private key failed, {0}")]
    LoadKey(IoError),
    #[error("parse private key failed, {0}")]
    ParseKey(IoError),
    #[error("tls error, {0}")]
    TlsErr(#[from] TlsError),
}
