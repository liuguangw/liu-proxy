use super::app_command::AppCommand;
use crate::{rt, services::proxy_client};
use clap::Args;

/// 客户端命令
#[derive(Args)]
pub struct ProxyClientCommand {
    ///local socket5 ip
    #[clap(long, value_parser, default_value_t = String::from("127.0.0.1"))]
    address: String,
    ///local socket5 port
    #[clap(long, value_parser, default_value_t = 1071)]
    port: u16,
    ///remote server ip
    #[clap(long, value_parser, default_value_t = String::from("127.0.0.1"))]
    server_address: String,
    ///remote server port
    #[clap(long, value_parser, default_value_t = 1070)]
    server_port: u16,
}

impl AppCommand for ProxyClientCommand {
    fn execute(&self) {
        let listen_address = (self.address.as_str(), self.port);
        let server_address = (self.server_address.as_str(), self.server_port);
        let fut = proxy_client::execute(listen_address, server_address);
        if let Err(err) = rt::block_on(fut) {
            panic!("{}", err)
        }
    }
}
