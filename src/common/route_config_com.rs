use super::{geosite::DomainRuleGroup, route_config::RouteConfigAction};

///解析处理过的路由配置
#[derive(Debug)]
pub struct RouteConfigCom {
    pub default_action: RouteConfigAction,
    pub rules: Vec<RouteConfigRuleCom>,
}
#[derive(Debug)]
pub struct RouteConfigRuleCom {
    pub t_action: RouteConfigAction,
    pub selection: DomainRuleGroup,
}

impl RouteConfigCom {
    pub fn match_action(&self, conn_dest: &str) -> RouteConfigAction {
        let host = match conn_dest.find(':') {
            Some(pos) => &conn_dest[..pos],
            None => conn_dest,
        };
        for rule in &self.rules {
            if rule.selection.match_domain(host) {
                return rule.t_action;
            }
        }
        self.default_action
    }
}
