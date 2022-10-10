use super::AuthUser;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
///客户端配置
pub struct ClientConfig {
    ///本地监听地址
    pub address: String,
    ///本地监听端口
    pub port: u16,
    ///授权用户配置
    pub auth_user: AuthUser,
    ///服务端url
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
