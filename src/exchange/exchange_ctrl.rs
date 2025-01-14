use crate::exchange::exchange_svc;
use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Query;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MarketStatusParams {
    pub stock_code: String,
}

#[get("/market/status")]
async fn get_market_status_by_stock_code(
    Query(params): Query<MarketStatusParams>,
) -> impl IntoResponse {
    let r = exchange_svc::get_market_status_by_stock_code(&params.stock_code).await;
    RespBody::result(&r)
}
