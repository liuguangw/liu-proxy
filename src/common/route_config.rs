use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RouteConfig {
    pub default_action: RouteConfigAction,
    pub rules: Vec<RouteConfigRule>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum RouteConfigAction {
    #[serde(rename(deserialize = "direct"))]
    Direct,
    #[serde(rename(deserialize = "proxy"))]
    Proxy,
    #[serde(rename(deserialize = "block"))]
    Block,
}

#[derive(Deserialize, Debug)]
pub struct RouteConfigRule {
    #[serde(rename(deserialize = "action"))]
    pub t_action: RouteConfigAction,
    pub selection: Vec<String>,
}
