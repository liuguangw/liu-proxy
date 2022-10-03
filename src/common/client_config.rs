use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
///客户端配置
pub struct ClientConfig {
    pub address: String,
    pub port: u16,
    pub auth_token: String,
    pub server_url: String,
    ///建立ssl连接时,是否跳过ssl证书验证
    pub insecure: bool,
    ///指定ip或者host来建立tcp连接
    pub server_host: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 8002,
            auth_token: String::default(),
            server_url: String::default(),
            insecure: false,
            server_host: String::default(),
        }
    }
}
