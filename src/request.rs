use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use reqwest::Client;

lazy_static! {
    pub static ref CLIENT: Arc<RwLock<Client>> = {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        Arc::new(RwLock::new(client))
    };
}

pub async fn client() -> std::sync::RwLockReadGuard<'static, Client> {
    CLIENT.read().unwrap()
}
