use super::app_command::AppCommand;
use crate::{rt, services::proxy_client};
use clap::Args;

/// 客户端命令
#[derive(Args)]
pub struct ProxyClientCommand {
    ///local socket5 ip
    #[clap(long, short = 'H', value_parser, default_value_t = String::from("127.0.0.1"))]
    address: String,
    ///local socket5 port
    #[clap(long, short = 'P', value_parser, default_value_t = 1071)]
    port: u16,
    ///server url
    #[clap(long, short = 'S', value_parser, default_value_t = String::from("ws://127.0.0.1:1070"))]
    server_url: String,
    ///custom host or ip for tcp connection
    #[clap(long, value_parser)]
    server_host: Option<String>,
}

impl AppCommand for ProxyClientCommand {
    fn execute(&self) {
        let listen_address = (self.address.as_str(), self.port);
        let fut = proxy_client::execute(listen_address, &self.server_url, &self.server_host);
        if let Err(err) = rt::block_on(fut) {
            panic!("{}", err)
        }
    }
}
