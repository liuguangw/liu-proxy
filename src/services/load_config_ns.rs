use crate::common::ConfigError;
use serde::de::DeserializeOwned;
use tokio::fs;
use toml::Value;

///加载配置文件
pub async fn load_config<'a, T: DeserializeOwned>(
    config_file: &str,
    section: &'a str,
) -> Result<T, ConfigError<'a>> {
    let content = fs::read_to_string(config_file).await?;
    let config_all: Value = toml::from_str(&content)?;
    let section_value = match config_all.get(section) {
        Some(s) => s.to_owned(),
        None => return Err(ConfigError::Section(section)),
    };
    let config = section_value.try_into()?;
    Ok(config)
}
