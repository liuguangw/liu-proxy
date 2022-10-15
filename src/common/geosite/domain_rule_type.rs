use serde_repr::{Deserialize_repr, Serialize_repr};

///域名匹配规则类型
#[derive(Debug, PartialEq, Eq, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DomainRuleType {
    Include,
    Domain,
    Keyword,
    Regexp,
    Full,
}
