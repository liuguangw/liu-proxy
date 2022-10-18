use serde::Deserialize;

///路由配置
#[derive(Deserialize, Debug)]
pub struct RouteConfig {
    ///域名默认路由行为
    pub default_domain_action: RouteConfigAction,
    ///ip默认路由行为
    pub default_ip_action: RouteConfigAction,
    ///域名规则列表
    pub domain_rules: Vec<RouteConfigRule>,
    ///ip规则列表
    pub ip_rules: Vec<RouteConfigRule>,
}

///路由行为
#[derive(Deserialize, Debug, Clone, Copy)]
pub enum RouteConfigAction {
    ///直连
    #[serde(rename(deserialize = "direct"))]
    Direct,
    ///代理
    #[serde(rename(deserialize = "proxy"))]
    Proxy,
    ///阻断
    #[serde(rename(deserialize = "block"))]
    Block,
}

///路由规则配置
#[derive(Deserialize, Debug)]
pub struct RouteConfigRule {
    ///路由行为
    #[serde(rename(deserialize = "action"))]
    pub t_action: RouteConfigAction,
    ///一系列的域名匹配,或者IP匹配
    pub selection: Vec<String>,
}
