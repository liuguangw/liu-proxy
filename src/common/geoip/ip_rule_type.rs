///ip规则类型
pub enum IpRuleType {
    ///cidr格式,例如 `192.168.1.0/24`
    Cidr,
    ///国家/地区代码,例如 `cn`, `us`, `jp` 等
    CountryCode,
}
