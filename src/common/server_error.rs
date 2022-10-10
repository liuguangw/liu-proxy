use hyper::Error as HyperError;
use std::io::Error as IoError;
use thiserror::Error;

///启动服务端的错误
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("ssl cert path not set")]
    ConfigSSlCertNone,
    #[error("ssl key path not set")]
    ConfigSSlKeyNone,
    #[error("load ssl cert failed: {0}")]
    Cert(IoError),
    #[error("parse socket address failed: {0}")]
    ParseAddress(IoError),
    #[error("bind address {0} failed: {1}")]
    Bind(String, HyperError),
    #[error("run http service failed: {0}")]
    HttpService(HyperError),
    #[error("run http service failed: {0}")]
    HttpTlsService(IoError),
}
