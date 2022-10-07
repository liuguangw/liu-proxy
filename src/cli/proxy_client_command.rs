use super::app_command::AppCommand;
use crate::{rt, services::proxy_client};
use clap::Args;

/// 客户端命令
#[derive(Args)]
pub struct ProxyClientCommand {
    ///config file path
    #[clap(long, short = 'f', value_parser, default_value_t = String::from("./config/config.toml"))]
    config_file: String,
}

impl AppCommand for ProxyClientCommand {
    fn execute(&self) {
        let fut = proxy_client::execute(&self.config_file);
        if let Err(err) = rt::block_on(fut) {
            log::error!("{}", err);
        }
    }
}
