use thiserror::Error;
use toml::de::Error as ParseError;

///配置加载错误
///
/// 将配置文件解析为 [`super::ServerConfig`] 或者 [`super::ClientConfig`] 出现的错误
#[derive(Error, Debug)]
pub enum ConfigError {
    ///文件读取错误
    #[error("load file failed, {0}")]
    Io(#[from] std::io::Error),
    ///文件解析错误
    #[error("parse failed, {0}")]
    Parse(#[from] ParseError),
    ///section 不存在
    #[error("section {0} not found")]
    Section(&'static str),
}
