use std::time::SystemTime;

use crate::{
    common::{ClientError, RouteConfig, RouteConfigCom, RouteConfigRuleCom},
    services::{self, geosite},
};

pub async fn load_route_config(
    route_file: &str,
    data_dir: &str,
) -> Result<RouteConfigCom, ClientError> {
    //加载geosite数据
    let time_1 = SystemTime::now();
    let geosite_data_path = format!("{data_dir}/geosite.pak");
    log::info!("load geosite data");
    let geosite_data = geosite::from_binary_file(&geosite_data_path).await?;
    let time_2 = SystemTime::now();
    let d = time_2.duration_since(time_1).unwrap();
    log::info!("load geosite data ok {d:?}");
    log::info!("parse routes {route_file}");
    let routes_config: RouteConfig = services::load_config(route_file, "client")
        .await
        .map_err(|e| ClientError::Config(route_file.to_string(), e))?;
    dbg!(&routes_config);
    let mut route_config_com = RouteConfigCom {
        default_action: routes_config.default_action,
        rules: Vec::default(),
    };
    let time_1 = SystemTime::now();
    for rule in routes_config.rules {
        let selection = geosite::parse_route_selection(&rule.selection, &geosite_data)?;
        let t_action = rule.t_action;
        route_config_com.rules.push(RouteConfigRuleCom {
            t_action,
            selection,
        });
    }
    let time_2 = SystemTime::now();
    let d = time_2.duration_since(time_1).unwrap();
    log::info!("parse selection list {d:?}");
    Ok(route_config_com)
}
