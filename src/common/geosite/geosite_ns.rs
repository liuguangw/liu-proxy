use super::DomainRule;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

///规则文件的对象结构
#[derive(Debug, Serialize, Deserialize)]
pub struct GeoSite {
    ///所有规则列表
    pub all_rules: Vec<DomainRule>,
    ///文件=>id映射
    pub file_rules: HashMap<String, HashSet<usize>>,
}
