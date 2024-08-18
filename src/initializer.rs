use crate::analysis::stock_analysis_ctrl::stock_analysis_routers;
use crate::currency::currency_ctrl::currency_routers;
use crate::debt::debt_ctrl::debt_routers;
use crate::holiday::holiday_ctrl::holiday_routers;
use crate::index::stock_index_ctrl::stock_index_routers;
use crate::stock::stock_ctrl::stock_routers;
use application::initializer::ServletContextInitializer;
use axum::Router;
use web::health::health_routers;

pub struct RoutInitializer {}

impl ServletContextInitializer for RoutInitializer {
    fn initialize(&self, router: Router) -> Router {
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
