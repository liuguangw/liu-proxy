use super::app_command::AppCommand;
use crate::{rt, services::geosite};
use clap::Args;

/// 构建geosite文件命令
#[derive(Args)]
pub struct BuildGeositeCommand {
    ///input dir path
    #[clap(long = "input", short = 'i', value_parser, default_value_t = String::from("./geo_data_src"))]
    input_dir: String,
    ///output file path
    #[clap(long = "output", short = 'o', value_parser, default_value_t = String::from("./data/geosite.pak"))]
    output_file: String,
}

impl AppCommand for BuildGeositeCommand {
    fn execute(&self) {
        let fut = Self::build(&self.input_dir, &self.output_file);
        if let Err(err) = rt::block_on(fut) {
            log::error!("{}", err);
        }
    }
}

impl BuildGeositeCommand {
    async fn build(input_dir: &str, output_file: &str) -> Result<(), String> {
        let geosite_data = geosite::from_source_dir(input_dir)
            .await
            .map_err(|e| e.to_string())?;
        geosite::save_as_binary(output_file, &geosite_data)
            .await
            .map_err(|e| e.to_string())
    }
}
