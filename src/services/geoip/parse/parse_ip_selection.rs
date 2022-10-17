use crate::common::geoip::{IpRuleGroup, IpRuleType};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseIpSelectionError {
    #[error("invalid type {0}")]
    InvalidType(String),
}

///解析路由配置中的selection字段选择的ip规则
pub fn parse_ip_selection(selection_list: &[String]) -> Result<IpRuleGroup, ParseIpSelectionError> {
    let mut rule_group = IpRuleGroup::default();
    for selection_node in selection_list {
        let (rule_type, value) = parse_route_selection_node(selection_node)?;
        rule_group.add_rule(rule_type, value);
    }
    Ok(rule_group)
}

fn parse_route_selection_node(
    selection_node: &str,
) -> Result<(IpRuleType, String), ParseIpSelectionError> {
    let (t_type, s_text) = match selection_node.find(':') {
        Some(s) => {
            let t_type_text = selection_node[..s].trim();
            let s_text = selection_node[s + 1..].trim();
            if t_type_text == "geoip" {
                (IpRuleType::CountryCode, s_text)
            } else {
                return Err(ParseIpSelectionError::InvalidType(t_type_text.to_string()));
            }
        }
        //默认为cidr ip
        None => (IpRuleType::Cidr, selection_node.trim()),
    };
    Ok((t_type, s_text.to_string()))
}
