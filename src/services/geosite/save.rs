use crate::common::geosite::GeoSite;
use rmp_serde::encode::Error as SerializeError;
use std::io::Error as IoError;
use thiserror::Error;
use tokio::fs;

///保存二进制文件的错误
#[derive(Error, Debug)]
pub enum SaveBinaryError {
    #[error("save file failed: {0}")]
    SaveFileErr(#[from] IoError),
    #[error("serialize failed: {0}")]
    SerializeErr(#[from] SerializeError),
}

///保存GeoSite为二进制文件
pub async fn save_as_binary(path: &str, geosite_data: &GeoSite) -> Result<(), SaveBinaryError> {
    let data = rmp_serde::to_vec(geosite_data)?;
    fs::write(path, &data).await?;
    Ok(())
}
