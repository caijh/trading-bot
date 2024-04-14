use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use web::response::RespBody;

use crate::stock_svc::{get_stock_daily_price, get_stock_price, init_stocks};

pub fn stock_routers() -> Router {
    Router::new()
        .route("/init", get(init))
        .route("/:code/daily", get(stock_daily))
        .route("/:code/price", get(stock_price))
}

pub async fn init() -> impl IntoResponse {
    let r = init_stocks().await;
    RespBody::from_result(&r).response()
}

pub async fn stock_daily(Path(code): Path<String>) -> impl IntoResponse {
    let r = get_stock_daily_price(&code).await;
    RespBody::from_result(&r).response()
}

pub async fn stock_price(Path(code): Path<String>) -> impl IntoResponse {
    let r = get_stock_price(&code).await;
    RespBody::from_result(&r).response()
}
