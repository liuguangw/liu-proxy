use super::{geoip::IpRuleGroup, geosite::DomainRuleGroup, route_config::RouteConfigAction};
use maxminddb::Reader;
use regex::Regex;
use std::net::IpAddr;

///解析处理过的路由配置
#[derive(Debug)]
pub struct RouteConfigCom {
    pub default_domain_action: RouteConfigAction,
    pub default_ip_action: RouteConfigAction,
    pub domain_rules: Vec<RouteConfigDomainRuleCom>,
    pub ip_rules: Vec<RouteConfigIpRuleCom>,
    pub mmdb_data: Option<Reader<Vec<u8>>>,
}
#[derive(Debug)]
pub struct RouteConfigDomainRuleCom {
    pub t_action: RouteConfigAction,
    pub selection: DomainRuleGroup,
}

#[derive(Debug)]
pub struct RouteConfigIpRuleCom {
    pub t_action: RouteConfigAction,
    pub selection: IpRuleGroup,
}

impl Default for RouteConfigCom {
    fn default() -> Self {
        Self {
            default_domain_action: RouteConfigAction::Proxy,
            default_ip_action: RouteConfigAction::Proxy,
            domain_rules: Vec::default(),
            ip_rules: Vec::default(),
            mmdb_data: None,
        }
    }
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
        if is_domain {
            self.match_domain(host)
        } else {
            self.match_ip(host)
        }
    }

    fn match_domain(&self, domain: &str) -> RouteConfigAction {
        for rule in &self.domain_rules {
            if rule.selection.match_domain(domain) {
                return rule.t_action;
            }
        }
        self.default_domain_action
    }

    fn match_ip(&self, ip_str: &str) -> RouteConfigAction {
        let ip_addr: IpAddr = match ip_str.parse() {
            Ok(s) => s,
            Err(e) => {
                log::error!("parse ip {ip_str} failed: {e}");
                return self.default_ip_action;
            }
        };
        if let Some(mmdb_data) = &self.mmdb_data {
            for rule in &self.ip_rules {
                if rule.selection.match_ip(&ip_addr, mmdb_data) {
                    return rule.t_action;
                }
            }
        }
        self.default_ip_action
    }
}
