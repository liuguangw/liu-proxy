use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
///服务端配置
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub auth_tokens: Vec<String>,
    pub use_ssl: bool,
    pub public_key_path: String,
    pub private_key_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".to_string(),
            port: 7008,
            auth_tokens: Vec::new(),
            use_ssl: false,
            public_key_path: String::default(),
            private_key_path: String::default(),
        }
    }
}
