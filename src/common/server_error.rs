use super::ConfigError;
use rustls::Error as TlsError;
use std::io::Error as IoError;
use thiserror::Error;

///运行服务端的错误
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("load {0} failed: {1}")]
    Config(String, ConfigError),
    #[error("wait signal failed: {0}")]
    Signal(IoError),
    #[error("load ssl cert failed: {0}")]
    Cert(#[from] TlsServerConfigError),
    #[error("bind address failed: {0}")]
    Bind(IoError),
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
