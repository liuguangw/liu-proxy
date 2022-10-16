use crate::common::geosite::{DomainRule, DomainRuleType, GeoSite, ParseDomainRuleError};
use std::{
    collections::{HashMap, HashSet},
    io::Error as IoError,
    path::Path,
};
use thiserror::Error;
use tokio::fs;

///从源文件加载的错误
#[derive(Error, Debug)]
pub enum FromSourceError {
    #[error("read dir {0} failed: {1}")]
    ReadDir(String, IoError),
    #[error("get file type failed: {0}")]
    LoadFileType(IoError),
    #[error("parse file name failed")]
    FileNameUtf8,
    #[error("read file {0} failed: {1}")]
    ReadFile(String, IoError),
    #[error("parse domain rule failed: {0}")]
    ParseRule(ParseDomainRuleError),
    #[error("include {0} not found")]
    IncludeNotFound(String),
}

///从源文件夹加载GeoSite
pub async fn from_source_dir(source_dir: &str) -> Result<GeoSite, FromSourceError> {
    let source_rules_map = load_source_rules_map(source_dir).await?;
    let file_count = source_rules_map.len();
    let mut rules_map = HashMap::with_capacity(file_count);
    for (index, file_name) in source_rules_map.keys().enumerate() {
        //依次解析每个文件里面的include指令
        log::info!("resolve [{}/{file_count}]{file_name}", index + 1);
        resolve_rules(&source_rules_map, file_name, &mut rules_map)?;
    }
    //
    //
    let mut all_rules = Vec::new();
    let mut insert_rule_fn = |rule| {
        //如果存在,则返回索引位置
        for (i, item) in all_rules.iter().enumerate() {
            if item == &rule {
                return i;
            }
        }
        //push之后返回位置
        all_rules.push(rule);
        all_rules.len() - 1
    };
    let mut file_rules_map = HashMap::new();
    let mut index = 0;
    for (file_name, v_rules) in rules_map {
        index += 1;
        log::info!("parse [{}/{file_count}]{file_name}", index + 1);
        let mut file_rules_set = HashSet::new();
        for rule in v_rules {
            let rule_id = insert_rule_fn(rule);
            file_rules_set.insert(rule_id);
        }
        file_rules_map.insert(file_name.to_string(), file_rules_set);
    }
    Ok(GeoSite {
        all_rules,
        file_rules: file_rules_map,
    })
}

///加载所有文件中的规则列表,不处理include
async fn load_source_rules_map(
    source_dir: &str,
) -> Result<HashMap<String, Vec<DomainRule>>, FromSourceError> {
    let mut entries = fs::read_dir(source_dir)
        .await
        .map_err(|e| FromSourceError::ReadDir(source_dir.to_string(), e))?;
    let mut source_rules_map = HashMap::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| FromSourceError::ReadDir(source_dir.to_string(), e))?
    {
        let f_path = entry.path();
        let f_type = entry
            .file_type()
            .await
            .map_err(FromSourceError::LoadFileType)?;
        if f_type.is_file() {
            let file_name = match f_path.file_name().unwrap().to_str() {
                Some(s) => s,
                None => return Err(FromSourceError::FileNameUtf8),
            };
            log::info!("load {file_name}");
            let source_rules = load_source_rules(&f_path, file_name).await?;
            source_rules_map.insert(file_name.to_string(), source_rules);
        }
    }
    Ok(source_rules_map)
}

///加载源文件中的规则列表,不处理include
async fn load_source_rules(
    path: impl AsRef<Path>,
    file_name: &str,
) -> Result<Vec<DomainRule>, FromSourceError> {
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| FromSourceError::ReadFile(file_name.to_string(), e))?;
    let mut rules = Vec::new();
    for line in content.split('\n') {
        let rule = match line.parse::<DomainRule>() {
            Ok(s) => s,
            Err(e) => match e {
                ParseDomainRuleError::Empty => continue,
                _ => return Err(FromSourceError::ParseRule(e)),
            },
        };
        rules.push(rule);
    }
    Ok(rules)
}

///解析rules,处理include
fn resolve_rules(
    source_rules_map: &HashMap<String, Vec<DomainRule>>,
    file_name: &str,
    rules_map: &mut HashMap<String, Vec<DomainRule>>,
) -> Result<(), FromSourceError> {
    if rules_map.contains_key(file_name) {
        return Ok(());
    }
    let src_rules = match source_rules_map.get(file_name) {
        Some(s) => s.as_slice(),
        None => return Err(FromSourceError::IncludeNotFound(file_name.to_string())),
    };
    let mut dest_rules = Vec::with_capacity(src_rules.len());
    for rule in src_rules {
        if rule.rule_type == DomainRuleType::Include {
            let sub_file_name = rule.value.as_str();
            resolve_rules(source_rules_map, sub_file_name, rules_map)?;
            let sub_rules = rules_map.get(sub_file_name).unwrap();
            let sub_attrs_filter = rule.attrs.as_ref();
            for sub_rule in sub_rules {
                if sub_rule.match_attrs(sub_attrs_filter) {
                    dest_rules.push(sub_rule.to_owned());
                }
            }
        } else {
            dest_rules.push(rule.to_owned());
        }
    }
    rules_map.insert(file_name.to_string(), dest_rules);
    Ok(())
}
