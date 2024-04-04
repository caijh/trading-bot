use axum::extract::Path;
use axum::response::IntoResponse;
use web::response::RespBody;

use crate::stock::init_stocks;
use crate::stock_data::get_stock_daily_price;

pub async fn init() -> impl IntoResponse {
    let r = init_stocks().await;
    RespBody::from_result(&r).response()
}

pub async fn stock_daily(Path(code): Path<String>) -> impl IntoResponse {
    let r = get_stock_daily_price(&code).await;
    RespBody::from_result(&r).response()
}