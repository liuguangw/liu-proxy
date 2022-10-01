use super::{
    app_command::AppCommand, hello_command::HelloCommand, proxy_client_command::ProxyClientCommand,
    proxy_server_command::ProxyServerCommand,
};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    #[clap(name = "hello", about = "hello world command")]
    Hello(HelloCommand),
    #[clap(name = "server", about = "proxy server command")]
    ProxyServer(ProxyServerCommand),
    #[clap(name = "client", about = "proxy client command")]
    ProxyClient(ProxyClientCommand),
}

impl AppCommand for Commands {
    fn execute(&self) {
        match self {
            Self::Hello(s) => s.execute(),
            Self::ProxyServer(s) => s.execute(),
            Self::ProxyClient(s) => s.execute(),
        }
    }
}
