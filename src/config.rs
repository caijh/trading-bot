use std::path::Path;
use config::{Config, ConfigBuilder};
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppConfig {

}

lazy_static! {
    static ref CONFIG: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(AppConfig::default()));
}

impl AppConfig {

}

pub fn load_app_config(file: &str) {
    let settings = Config::builder()
        .add_source(Path::new(file))
        .build()
        .expect(format!("[!] Fail to load config file {}", file).as_str());
    let cfg = settings.try_deserialize::<AppConfig>().unwrap();
    let config_clone = CONFIG.clone();
    let mut config = config_clone.write().unwrap();
    *config = cfg;
}

pub fn get_app_config() -> std::sync::RwLockReadGuard<'static, AppConfig> {
    CONFIG.read().unwrap()
}
