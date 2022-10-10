use super::AuthUser;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
///服务端配置
pub struct ServerConfig {
    ///地址
    pub address: String,
    ///端口
    pub port: u16,
    ///`url` 中的 `path` 部分
    pub path: String,
    ///授权用户列表
    pub auth_users: Vec<AuthUser>,
    ///是否启用ssl
    pub use_ssl: bool,
    ///ssl证书路径
    pub ssl_cert_path: Option<String>,
    ///ssl密钥路径
    pub ssl_key_path: Option<String>,
    ///工作线程数量
    pub worker_count: Option<usize>,
}
