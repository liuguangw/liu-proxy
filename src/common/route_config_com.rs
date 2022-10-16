use std::net::IpAddr;

use regex::Regex;

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
        let pos = conn_dest.rfind(':').unwrap();
        let host = &conn_dest[..pos];
        let is_domain = {
            if host.contains(':') {
                //ipv6
                false
            } else {
                match host.rfind('.') {
                    Some(pos) => {
                        let root_domain = &host[pos + 1..];
                        let num_rexp = Regex::new("^\\d+$").unwrap();
                        !num_rexp.is_match(root_domain)
                    }
                    None => true,
                }
            }
        };
        if !is_domain {
            //log::info!("[is_ip]{host}");
            let ip_addr: IpAddr = match host.parse() {
                Ok(s) => s,
                Err(e) => {
                    log::error!("parse ip {host} failed: {e}");
                    return self.default_action;
                }
            };
            let need_direct = match ip_addr {
                IpAddr::V4(s) => {
                    s.is_private() || s.is_loopback() || s.is_broadcast() || s.is_multicast()
                }
                IpAddr::V6(s) => s.is_loopback() || s.is_multicast(),
            };
            if need_direct {
                return RouteConfigAction::Direct;
            }
            return RouteConfigAction::Proxy;
        }
        for rule in &self.rules {
            if rule.selection.match_domain(host) {
                return rule.t_action;
            }
        }
        self.default_action
    }
}
