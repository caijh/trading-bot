use async_trait::async_trait;
use axum::Router;
use config::Config;
use context::SERVICES;
use database::DbService;
use logger::Logger;
use web::bootstrap::Bootstrap;
use web::health::health_routers;

use crate::stock_ctrl::stock_routers;

pub struct StockBotServer;


#[async_trait]
impl Bootstrap for StockBotServer {
    async fn init(&self, config: &Config) {
        // set logger.
        Logger::init_logger(config);

        let database_service = DbService::create(config).await;
        SERVICES.set(database_service);
    }

    async fn init_routes(&self, router: Router) -> Router {
        // route
        router
            .nest("/health", health_routers())
            .nest("/stock", stock_routers())
    }
}
