use crate::common::geosite::GeoSite;
use rmp_serde::decode::Error as DeSerializeError;
use std::{io::Error as IoError, path::Path};
use thiserror::Error;
use tokio::fs;

///从二进制文件加载的错误
#[derive(Error, Debug)]
pub enum FromBinaryError {
    #[error("read file failed: {0}")]
    ReadFile(#[from] IoError),
    #[error("deserialize file failed: {0}")]
    DeSerializeErr(#[from] DeSerializeError),
}

///从二进制文件加载GeoSite
pub async fn from_binary_file(path: &Path) -> Result<GeoSite, FromBinaryError> {
    let file_data = fs::read(path).await?;
    let geosite_data = rmp_serde::from_slice(&file_data)?;
    Ok(geosite_data)
}
