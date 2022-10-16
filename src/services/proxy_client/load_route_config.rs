use std::{path::PathBuf, time::SystemTime};

use crate::{
    common::{ClientError, RouteConfig, RouteConfigCom, RouteConfigRuleCom},
    services::{self, geosite},
};

pub async fn load_route_config(
    route_file: &str,
    data_dir: &str,
) -> Result<RouteConfigCom, ClientError> {
    let geosite_data_path = PathBuf::from(format!("{data_dir}/geosite.pak"));
    if !geosite_data_path.exists() {
        return Ok(RouteConfigCom::default());
    }
    //加载geosite数据
    let time_1 = SystemTime::now();
    log::info!("load geosite data");
    let geosite_data = geosite::from_binary_file(&geosite_data_path).await?;
    let time_2 = SystemTime::now();
    let d = time_2.duration_since(time_1).unwrap();
    log::info!("load geosite data ok {d:?}");
    log::info!("parse routes {route_file}");
    let routes_config: RouteConfig = services::load_config(route_file, "client")
        .await
        .map_err(|e| ClientError::Config(route_file.to_string(), e))?;
    //dbg!(&routes_config);
    let mut route_config_com = RouteConfigCom {
        default_domain_action: routes_config.default_domain_action,
        default_ip_action: routes_config.default_ip_action,
        domain_rules: Vec::default(),
    };
    let time_1 = SystemTime::now();
    for rule in routes_config.domain_rules {
        let selection = geosite::parse_domain_selection(&rule.selection, &geosite_data)?;
        let t_action = rule.t_action;
        route_config_com.domain_rules.push(RouteConfigRuleCom {
            t_action,
            selection,
        });
    }
    let time_2 = SystemTime::now();
    let d = time_2.duration_since(time_1).unwrap();
    log::info!("parse selection list {d:?}");
    Ok(route_config_com)
}
