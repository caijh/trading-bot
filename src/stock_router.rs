use axum::Router;
use axum::routing::get;

use crate::stock_ctrl::init;

pub fn stock_routers() -> Router {
    Router::new()
        .route("/init", get(init))
}


