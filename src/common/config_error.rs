use thiserror::Error;
use toml::de::Error as ParseError;

///配置加载错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("load file failed, {0}")]
    Io(#[from] std::io::Error),
    #[error("parse failed, {0}")]
    Parse(#[from] ParseError),
    #[error("section {0} not found")]
    Section(&'static str),
}
