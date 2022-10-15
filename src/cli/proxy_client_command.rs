use super::app_command::AppCommand;
use crate::{rt, services::proxy_client};
use clap::Args;

/// 客户端命令
#[derive(Args)]
pub struct ProxyClientCommand {
    ///config file path
    #[clap(long, short = 'f', value_parser, default_value_t = String::from("./config/config.toml"))]
    config_file: String,
    ///routes config file path
    #[clap(long, short = 'r', value_parser, default_value_t = String::from("./config/routes.toml"))]
    route_file: String,
    ///data dir path
    #[clap(long, value_parser, default_value_t = String::from("./data"))]
    data_dir: String,
}

impl AppCommand for ProxyClientCommand {
    fn execute(&self) {
        let fut = proxy_client::execute(&self.config_file, &self.route_file, &self.data_dir);
        if let Err(err) = rt::block_on(fut) {
            log::error!("{}", err);
        }
    }
}
