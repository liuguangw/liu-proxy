use super::app_command::AppCommand;
use clap::Args;

/// 打印hello world的命令
#[derive(Args)]
pub struct HelloCommand;

impl AppCommand for HelloCommand {
    fn execute(&self) {
        println!("hello world")
    }
}
