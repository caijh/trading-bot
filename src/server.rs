use async_trait::async_trait;
use axum::Router;
use config::Config;
use logger::{Logger, LoggerConfig};
use web::bootstrap::Bootstrap;
use web::health::health_routers;

pub struct StockBotServer;

#[async_trait]
impl Bootstrap for StockBotServer {
    async fn init(&self, config: &Config) {
        // set logger.
        let logger_config = LoggerConfig::get_config(config);
        Logger::init_logger(&logger_config);
    }

    async fn init_routes(&self, router: Router) -> Router {
        // route
        router
            .nest("/health", health_routers())
    }
}