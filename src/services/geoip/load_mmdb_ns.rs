use maxminddb::{MaxMindDBError, Reader};
use std::path::Path;
use tokio::task;

///加载mmdb
pub async fn load_mmdb(path: impl AsRef<Path>) -> Result<Reader<Vec<u8>>, MaxMindDBError> {
    let path = path.as_ref().to_owned();
    task::spawn_blocking(|| Reader::open_readfile(path))
        .await
        .expect("spawn_blocking load_mmdb failed")
}
