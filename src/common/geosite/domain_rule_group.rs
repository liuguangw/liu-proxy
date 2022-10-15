use regex::Regex;

use super::{DomainRule, DomainRuleType};

///代表选择的一组匹配规则
#[derive(Debug, Default)]
pub struct DomainRuleGroup {
    pub domain_list: Vec<DomainRule>,
    pub keyword_list: Vec<DomainRule>,
    pub regexp_list: Vec<DomainRule>,
    pub full_list: Vec<DomainRule>,
}

impl DomainRuleGroup {
    pub fn add_rule(&mut self, rule: DomainRule) {
        let coll = match &rule.rule_type {
            DomainRuleType::Include => panic!("invalid include"),
            DomainRuleType::Domain => &mut self.domain_list,
            DomainRuleType::Keyword => &mut self.keyword_list,
            DomainRuleType::Regexp => &mut self.regexp_list,
            DomainRuleType::Full => &mut self.full_list,
        };
        //if !coll.contains(&rule) {
            coll.push(rule);
        //}
    }
    pub fn merge_group(&mut self, group: DomainRuleGroup) {
        let src_list = [
            group.domain_list,
            group.keyword_list,
            group.regexp_list,
            group.full_list,
        ];
        for (index, from_list) in src_list.into_iter().enumerate() {
            let target_list = if index == 0 {
                &mut self.domain_list
            } else if index == 1 {
                &mut self.keyword_list
            } else if index == 2 {
                &mut self.regexp_list
            } else if index == 3 {
                &mut self.full_list
            } else {
                panic!("invalid index")
            };
            for rule in from_list {
                //if !target_list.contains(&rule) {
                    target_list.push(rule);
                //}
            }
        }
    }

    pub fn match_domain(&self, domain: &str) -> bool {
        //完全匹配
        for s in &self.full_list {
            if s.value == domain {
                return true;
            }
        }
        //域名匹配
        for s in &self.domain_list {
            if s.value == domain {
                return true;
            }
            let r_domain = if s.value.starts_with('.') {
                s.value.to_string()
            } else {
                format!(".{}", s.value)
            };
            if domain.ends_with(&r_domain) {
                return true;
            }
        }
        //关键词
        for s in &self.keyword_list {
            if domain.contains(&s.value) {
                return true;
            }
        }
        //正则表达式
        for s in &self.regexp_list {
            let re = match Regex::new(&s.value) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("rexp {} error: {e}", &s.value);
                    continue;
                }
            };
            if re.is_match(domain) {
                return true;
            }
        }
        false
    }
}
