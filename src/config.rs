use clap::{Parser, Subcommand};
use config::Config;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppConfig {}

impl AppConfig {
    pub fn load_app_config(file: &str) {
        let settings = Config::builder()
            .add_source(config::File::from(Path::new(file)))
            .build()
            .unwrap_or_else(|_| panic!("[!] Fail to load config file {}", file));
        let cfg = settings.try_deserialize::<AppConfig>().unwrap();
        let config_clone = CONFIG.clone();
        let mut config = config_clone.write().unwrap();
        *config = cfg;
    }

    pub fn get_app_config() -> std::sync::RwLockReadGuard<'static, AppConfig> {
        CONFIG.read().unwrap()
    }
}

lazy_static! {
    static ref CONFIG: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(AppConfig::default()));
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    subcommand: CliSubCommand,
}


#[derive(Subcommand,Debug)]
enum CliSubCommand {

}



