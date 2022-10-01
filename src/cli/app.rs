use super::app_command::AppCommand;
use super::commands::Commands;
use clap::Parser;

/// 命令行应用
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true), disable_version_flag(true))]
pub struct App {
    /// 子命令
    #[clap(subcommand)]
    command: Option<Commands>,
    ///Print version info and exit
    #[clap(short = 'V', long = "version", value_parser)]
    show_version: bool,
    ///Use verbose output
    #[clap(short, long, value_parser)]
    verbose: bool,
}

impl App {
    /// 命令行入口
    pub fn run() {
        let app: Self = Self::parse();
        if let Some(sub_command) = &app.command {
            sub_command.execute();
        }
    }
}
