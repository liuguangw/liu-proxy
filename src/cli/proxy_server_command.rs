use super::app_command::AppCommand;
use crate::services::proxy_server;
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
        //使用actix运行时
        let web_rt = actix_web::rt::System::new();
        if let Err(err) = web_rt.block_on(fut) {
            log::error!("{}", err);
        }
        //使用tokio运行时
        /*if let Err(err) = crate::rt::block_on(fut) {
            log::error!("{}", err);
        }*/
    }
}
