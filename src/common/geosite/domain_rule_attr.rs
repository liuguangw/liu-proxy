use serde::{Deserialize, Serialize};

///域名规则附加属性
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct DomainRuleAttr {
    pub name: String,
    pub enabled: bool,
}

impl DomainRuleAttr {
    pub fn new(name: String, enabled: bool) -> Self {
        Self { name, enabled }
    }
}
