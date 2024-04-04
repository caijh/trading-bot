use axum::Router;
use axum::routing::get;

use crate::stock_ctrl::{init, stock_daily};

pub fn stock_routers() -> Router {
    Router::new()
        .route("/init", get(init))
        .route("/:code/daily", get(stock_daily))
}


