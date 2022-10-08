use super::AuthUser;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
///客户端配置
pub struct ClientConfig {
    pub address: String,
    pub port: u16,
    pub auth_user: AuthUser,
    pub server_url: String,
    ///连接池最多空闲连接个数
    pub max_idle_conns: u32,
    ///指定ip来建立tcp连接
    pub server_ip: Option<String>,
    ///建立ssl连接时,是否跳过ssl证书验证
    pub insecure: Option<bool>,
    ///额外的http请求头
    pub extra_http_headers: Option<Vec<[String; 2]>>,
}
