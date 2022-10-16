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
        let (host, is_domain) = {
            let pos = conn_dest.rfind(':').unwrap();
            let host = &conn_dest[..pos];
            if host.contains(':') {
                //ipv6
                (host, false)
            } else {
                let r = Regex::new("^\\d+\\.{3}\\d+$").expect("load ipv4 regexp failed");
                if r.is_match(host) {
                    (host, false)
                } else {
                    (host, true)
                }
            }
        };
        if !is_domain {
            log::info!("[is_ip]{host}");
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
