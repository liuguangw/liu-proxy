use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
///客户端配置
pub struct ClientConfig {
    pub address: String,
    pub port: u16,
    pub auth_token: String,
    pub server_url: String,
    ///指定ip来建立tcp连接
    pub server_ip: Option<String>,
    ///建立ssl连接时,是否跳过ssl证书验证
    pub insecure: Option<bool>,
}
