use std::collections::HashSet;

use crate::common::geosite::{
    DomainRule, DomainRuleAttr, DomainRuleGroup, DomainRuleType, GeoSite,
};
use futures_util::future::Either;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseDomainSelectionError {
    #[error("invalid type {0}")]
    InvalidType(String),
    #[error("geosite:{0} not found")]
    GeoSiteNotFound(String),
}

///解析路由配置中的selection字段选择的域名
pub fn parse_domain_selection(
    selection_list: &[String],
    geosite_data: &GeoSite,
) -> Result<DomainRuleGroup, ParseDomainSelectionError> {
    let mut rule_group = DomainRuleGroup::default();
    for selection_node in selection_list {
        let rules = parse_route_selection_node(selection_node, geosite_data)?;
        match rules {
            Either::Left(s) => rule_group.add_rule(s),
            Either::Right(group) => rule_group.add_group(group),
        }
    }
    Ok(rule_group)
}

fn parse_route_selection_node(
    selection_node: &str,
    geosite_data: &GeoSite,
) -> Result<Either<DomainRule, DomainRuleGroup>, ParseDomainSelectionError> {
    let (t_type, s_text) = match selection_node.find(':') {
        Some(s) => {
            let t_type_text = selection_node[..s].trim();
            let s_text = selection_node[s + 1..].trim();
            if t_type_text == "domain" {
                (DomainRuleType::Domain, s_text)
            } else if t_type_text == "keyword" {
                (DomainRuleType::Keyword, s_text)
            } else if t_type_text == "regexp" {
                (DomainRuleType::Regexp, s_text)
            } else if t_type_text == "full" {
                (DomainRuleType::Full, s_text)
            } else if t_type_text == "geosite" {
                let group = parse_geosite(s_text, geosite_data)?;
                return Ok(Either::Right(group));
            } else {
                return Err(ParseDomainSelectionError::InvalidType(
                    t_type_text.to_string(),
                ));
            }
        }
        //默认为域名
        None => (DomainRuleType::Domain, selection_node.trim()),
    };
    let rule = DomainRule::new(t_type, s_text.to_string());
    Ok(Either::Left(rule))
}

fn parse_geosite(
    s_text: &str,
    geosite_data: &GeoSite,
) -> Result<DomainRuleGroup, ParseDomainSelectionError> {
    let (file_name, attr_filter) = match s_text.find('@') {
        Some(pos) => {
            let file_name = s_text[..pos].trim();
            let attr_text = s_text[pos + 1..].trim();
            let attr = if let Some(disabled_name) = attr_text.strip_prefix('!') {
                DomainRuleAttr::new(disabled_name.to_string(), false)
            } else {
                DomainRuleAttr::new(attr_text.to_string(), true)
            };
            let mut attrs = HashSet::new();
            attrs.insert(attr);
            (file_name, Some(attrs))
        }
        None => (s_text, None),
    };
    let rule_ids = match geosite_data.file_rules.get(file_name) {
        Some(s) => s,
        None => {
            return Err(ParseDomainSelectionError::GeoSiteNotFound(
                file_name.to_string(),
            ))
        }
    };
    let mut group = DomainRuleGroup::default();
    for rule_id in rule_ids {
        let rule = geosite_data.all_rules.get(*rule_id).unwrap();
        if rule.match_attrs(attr_filter.as_ref()) {
            group.add_rule(rule.to_owned());
        }
    }
    Ok(group)
}
