use async_trait::async_trait;
use axum::Router;
use config::Config;
use context::SERVICES;
use database::DbService;
use logger::Logger;
use web::bootstrap::Bootstrap;
use web::health::health_routers;

use crate::currency::currency_ctrl::currency_routers;
use crate::debt::debt_ctrl::debt_routers;
use crate::holiday::holiday_ctrl::holiday_routers;
use crate::index::stock_index_ctrl::stock_index_routers;
use crate::jobs::load_jobs;
use crate::stock::stock_ctrl::stock_routers;
use crate::stock_analysis_ctrl::stock_analysis_routers;

pub struct StockBotServer;

#[async_trait]
impl Bootstrap for StockBotServer {
    async fn init(&self, config: &Config) {
        // set logger.
        Logger::init_logger(config);

        SERVICES.set(DbService::create(config).await);

        load_jobs().await.expect("Fail to load jobs");
    }

    async fn init_routes(&self, router: Router) -> Router {
        // route
        router
            .nest("/health", health_routers())
            .nest("/holiday", holiday_routers())
            .nest("/currency", currency_routers())
            .nest("/debt", debt_routers())
            .nest("/index", stock_index_routers())
            .nest("/stock", stock_routers())
            .nest("/analysis", stock_analysis_routers())
    }
}
