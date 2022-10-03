use super::app_command::AppCommand;
use crate::{rt, services::proxy_server};
use clap::Args;

/// 服务端命令
#[derive(Args)]
pub struct ProxyServerCommand {
    ///config file path
    #[clap(long, short = 'f', value_parser, default_value_t = String::from("./config/config.toml"))]
    config_file: String,
}

impl AppCommand for ProxyServerCommand {
    fn execute(&self) {
        let fut = proxy_server::execute(&self.config_file);
        if let Err(err) = rt::block_on(fut) {
            eprintln!("{}", err);
        }
    }
}
