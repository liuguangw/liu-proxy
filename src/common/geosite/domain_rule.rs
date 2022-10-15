use super::{domain_rule_attr::DomainRuleAttr, DomainRuleType};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, str::FromStr};
use thiserror::Error;

///域名匹配规则
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct DomainRule {
    pub rule_type: DomainRuleType,
    pub value: String,
    pub attrs: Option<HashSet<DomainRuleAttr>>,
}

impl DomainRule {
    pub fn new(rule_type: DomainRuleType, value: String) -> Self {
        Self {
            rule_type,
            value,
            attrs: None,
        }
    }
    pub fn add_attr(&mut self, name: String, enabled: bool) {
        let attr = DomainRuleAttr::new(name, enabled);
        match self.attrs.as_mut() {
            Some(attrs) => {
                attrs.insert(attr);
            }
            None => {
                let mut attrs = HashSet::new();
                attrs.insert(attr);
                self.attrs = Some(attrs);
            }
        }
    }
    ///匹配其中一条即可
    pub fn match_attrs(&self, attrs_filter: Option<&HashSet<DomainRuleAttr>>) -> bool {
        let need_attrs = match attrs_filter {
            Some(s) if !s.is_empty() => s,
            //无属性筛选条件
            _ => return true,
        };
        for node_attr in need_attrs {
            //出现这条属性即匹配
            if node_attr.enabled {
                if self.contains_attr(&node_attr.name) {
                    return true;
                }
            } else {
                //不出现这条属性即匹配
                if self.not_contains_attr(&node_attr.name) {
                    return true;
                }
            }
        }
        false
    }

    fn contains_attr(&self, name: &str) -> bool {
        if let Some(self_attrs) = &self.attrs {
            for node_attr in self_attrs {
                if node_attr.enabled && node_attr.name == name {
                    return true;
                }
            }
        }
        false
    }

    fn not_contains_attr(&self, name: &str) -> bool {
        match &self.attrs {
            Some(self_attrs) => {
                for node_attr in self_attrs {
                    if node_attr.enabled && node_attr.name == name {
                        return false;
                    }
                }
                true
            }
            None => true,
        }
    }
}

///解析域名匹配规则的错误
#[derive(Error, Debug)]
pub enum ParseDomainRuleError {
    #[error("no rule found")]
    Empty,
    #[error("attr name is empty")]
    EmptyAttrName,
    #[error("invalid rule type {0}")]
    InvalidRuleType(String),
}

impl FromStr for DomainRule {
    type Err = ParseDomainRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseDomainRuleError::Empty);
        }
        //去除comment
        let s = match s.find('#') {
            Some(pos) => s[..pos].trim_end(),
            None => s,
        };
        if s.is_empty() {
            return Err(ParseDomainRuleError::Empty);
        }
        let (rule_type, s) = match s.find(':') {
            Some(pos) => {
                let rule_type = s[..pos].trim_end();
                let s = s[pos + 1..].trim_start();
                let rule_type = if rule_type == "include" {
                    DomainRuleType::Include
                } else if rule_type == "domain" {
                    DomainRuleType::Domain
                } else if rule_type == "keyword" {
                    DomainRuleType::Keyword
                } else if rule_type == "regexp" {
                    DomainRuleType::Regexp
                } else if rule_type == "full" {
                    DomainRuleType::Full
                } else {
                    return Err(ParseDomainRuleError::InvalidRuleType(rule_type.to_string()));
                };
                (rule_type, s)
            }
            None => (DomainRuleType::Domain, s),
        };
        let (value, attrs) = match s.find('@') {
            Some(pos) => {
                let value = s[..pos].trim_end();
                let attrs = parse_attrs(&s[pos + 1..])?;
                (value, Some(attrs))
            }
            None => (s, None),
        };
        Ok(Self {
            rule_type,
            value: value.to_string(),
            attrs,
        })
    }
}

fn parse_attrs(s: &str) -> Result<HashSet<DomainRuleAttr>, ParseDomainRuleError> {
    let mut attrs = HashSet::new();
    let mut attrs_text = s;
    while let Some(next_pos) = attrs_text.find('@') {
        let s_text = attrs_text[..next_pos].trim_end();
        let (name, enabled) = if let Some(attr_name) = s_text.strip_prefix('!') {
            (attr_name, false)
        } else {
            (s_text, true)
        };
        if name.is_empty() {
            return Err(ParseDomainRuleError::EmptyAttrName);
        }
        let attr = DomainRuleAttr::new(name.to_string(), enabled);
        attrs.insert(attr);
        attrs_text = &attrs_text[next_pos + 1..]
    }
    //最后一个
    let (name, enabled) = if let Some(attr_name) = attrs_text.strip_prefix('!') {
        (attr_name, false)
    } else {
        (attrs_text, true)
    };
    if name.is_empty() {
        return Err(ParseDomainRuleError::EmptyAttrName);
    }
    let attr = DomainRuleAttr::new(name.to_string(), enabled);
    attrs.insert(attr);
    Ok(attrs)
}
