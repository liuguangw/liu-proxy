use super::app_command::AppCommand;
use crate::{
    common::ServerConfig,
    rt,
    services::{self, proxy_server},
};
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
        //加载配置
        let config: ServerConfig = match services::load_config_sync(&self.config_file, "server") {
            Ok(s) => s,
            Err(e) => {
                log::error!("load {} failed: {e}", &self.config_file);
                return;
            }
        };
        let worker_count_opt = config.worker_count.to_owned();
        if let Some(ref worker_count) = worker_count_opt {
            log::info!("start {worker_count} workers")
        }
        let runtime = rt::runtime(worker_count_opt);
        let fut = proxy_server::execute(config);
        //使用tokio运行时
        if let Err(err) = runtime.block_on(fut) {
            log::error!("{}", err);
        }
    }
}
