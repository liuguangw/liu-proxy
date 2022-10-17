use super::IpRuleType;
use ipnet::IpNet;
use maxminddb::geoip2::Country;
use maxminddb::Reader;
use std::{collections::HashSet, net::IpAddr};

///代表选择的一组匹配规则
#[derive(Debug, Default)]
pub struct IpRuleGroup {
    pub cidr_list: HashSet<String>,
    pub country_code_list: HashSet<String>,
}

impl IpRuleGroup {
    pub fn add_rule(&mut self, rule_type: IpRuleType, value: String) {
        match rule_type {
            IpRuleType::Cidr => {
                self.cidr_list.insert(value);
            }
            IpRuleType::CountryCode => {
                if value == "private" {
                    self.add_private_cidr_list();
                } else {
                    self.country_code_list.insert(value);
                }
            }
        }
    }

    fn add_private_cidr_list(&mut self) {
        let cidr_list = super::private_cidr_list();
        for s in cidr_list {
            self.cidr_list.insert(s.to_string());
        }
    }

    pub fn match_ip(&self, address: &IpAddr, mmdb_data: &Reader<Vec<u8>>) -> bool {
        for cidr in &self.cidr_list {
            let cidr_net: IpNet = cidr.parse().unwrap();
            if cidr_net.contains(address) {
                return true;
            }
        }
        let country_info = match mmdb_data.lookup::<Country>(*address) {
            Ok(s) => s,
            Err(e) => {
                log::error!("lookup {address} in mmdb failed: {e}");
                return false;
            }
        };
        //获取国家代码
        let mut opt_country_code = None;
        if let Some(country) = country_info.country {
            opt_country_code = country.iso_code;
        }
        if let Some(country_code) = opt_country_code {
            for code in &self.country_code_list {
                //不区分大小写
                if country_code.eq_ignore_ascii_case(code) {
                    return true;
                }
            }
        }
        false
    }
}
