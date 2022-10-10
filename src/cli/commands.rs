use super::{
    app_command::AppCommand, hello_command::HelloCommand, proxy_client_command::ProxyClientCommand,
    proxy_server_command::ProxyServerCommand,
};
use chrono::Local;
use clap::Subcommand;
use env_logger::{fmt::Color, Builder, Env};
use log::Level;
use std::io::Write;

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
        Self::init_logger();
        //log::info!("starting up");
        match self {
            Self::Hello(s) => s.execute(),
            Self::ProxyServer(s) => s.execute(),
            Self::ProxyClient(s) => s.execute(),
        }
    }
}

impl Commands {
    ///初始化日志样式
    fn init_logger() {
        //如果没有使用 `RUST_LOG` 指定默认日志过滤级别, 则默认为 `info`
        let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
        builder.format(|buf, record| {
            let datetime = Local::now();
            let datetime_str = datetime.format("%F %T");
            //有颜色的level
            let level = record.level();
            let styled_level = buf.default_styled_level(level);
            //warn 和以上级别附带文件路径信息
            if level <= Level::Warn {
                let mut file_txt_style = buf.style();
                file_txt_style.set_bold(true).set_color(Color::Cyan);
                let file_path = file_txt_style.value(record.file().unwrap_or("-"));
                let file_line = file_txt_style.value(record.line().unwrap_or(0));
                writeln!(
                    buf,
                    "[{datetime_str}] {styled_level} {} (in {file_path} on line {file_line})",
                    record.args()
                )
            } else {
                writeln!(buf, "[{datetime_str}] {styled_level} {}", record.args())
            }
        });
        builder.init();
    }
}
