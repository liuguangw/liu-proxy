use super::AuthUser;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
///服务端配置
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub path: String,
    pub auth_users: Vec<AuthUser>,
    pub use_ssl: bool,
    pub ssl_cert_path: Option<String>,
    pub ssl_key_path: Option<String>,
}
