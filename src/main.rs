use std::error::Error;

use clap::{command, Parser};
use stock_bot::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let args = Args::parse();
    let config_file_path  = if let Some(config_path) = args.config_path {
        config_path
    } else {
        "Config.toml".to_string()
    };
    AppConfig::load_app_config(&config_file_path);
    let app_config = AppConfig::get_app_config();
    println!("{:?}", app_config);
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, required = false)]
    pub config_path: Option<String>,
}
