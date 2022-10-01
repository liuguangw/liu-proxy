use super::app_command::AppCommand;
use crate::{rt, services::proxy_server};
use clap::Args;

/// 服务端命令
#[derive(Args)]
pub struct ProxyServerCommand {
    #[clap(long, value_parser, default_value_t = String::from("127.0.0.1"))]
    address: String,
    #[clap(long, value_parser, default_value_t = 1070)]
    port: u16,
}

impl AppCommand for ProxyServerCommand {
    fn execute(&self) {
        let fut = proxy_server::execute(&self.address, self.port);
        if let Err(err) = rt::block_on(fut) {
            panic!("{}", err)
        }
    }
}
