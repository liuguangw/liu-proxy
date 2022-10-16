use crate::common::geosite::GeoSite;
use rmp_serde::encode::Error as SerializeError;
use std::{io::Error as IoError, path::PathBuf};
use thiserror::Error;
use tokio::fs;

///保存二进制文件的错误
#[derive(Error, Debug)]
pub enum SaveBinaryError {
    #[error("save file failed: {0}")]
    SaveFileErr(IoError),
    #[error("create dir failed: {0}")]
    CreateDirErr(IoError),
    #[error("serialize failed: {0}")]
    SerializeErr(#[from] SerializeError),
}

///保存GeoSite为二进制文件
pub async fn save_as_binary(path: &str, geosite_data: &GeoSite) -> Result<(), SaveBinaryError> {
    let file_path = PathBuf::from(path);
    let path = file_path.as_path();
    //目录不存在,创建目录
    let dir_path = path.parent().unwrap();
    if !dir_path.exists() {
        fs::create_dir(dir_path)
            .await
            .map_err(SaveBinaryError::CreateDirErr)?;
    }
    let data = rmp_serde::to_vec(geosite_data)?;
    fs::write(path, &data)
        .await
        .map_err(SaveBinaryError::SaveFileErr)?;
    Ok(())
}
