use std::str::FromStr;

use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use web::response::RespBody;

use crate::analysis::stock_pattern::get_stock_pattern;
use crate::exchange::exchange_model::Exchange;
use crate::stock::stock_svc::{get_stock_daily_price, get_stock_price, sync_stocks};

pub fn stock_routers() -> Router {
    Router::new()
        .route("/sync/:exchange", get(sync))
        .route("/daily", get(stock_daily))
        .route("/price", get(stock_price))
        .route("/pattern", get(stock_pattern))
}

#[derive(Serialize, Deserialize)]
struct StockParams {
    code: String,
}

/**
 * 同步指定交易所的股票数据。
 *
 * # 参数
 * `exchange`: 代表需要同步的交易所的名称, sh or sz.
 *
 * # 返回值
 * 实现了 `IntoResponse` 的一个类型，通常用于构建HTTP响应。
 */
async fn sync(Path(exchange): Path<String>) -> impl IntoResponse {
    let exchange = Exchange::from_str(&exchange).unwrap();

    let r = sync_stocks(&exchange).await;

    RespBody::from_result(&r).response()
}

async fn stock_daily(Query(params): Query<StockParams>) -> impl IntoResponse {
    let r = get_stock_daily_price(&params.code).await;
    RespBody::from_result(&r).response()
}

async fn stock_price(Query(params): Query<StockParams>) -> impl IntoResponse {
    let r = get_stock_price(&params.code).await;
    RespBody::from_result(&r).response()
}

async fn stock_pattern(Query(params): Query<StockParams>) -> impl IntoResponse {
    let r = get_stock_daily_price(&params.code).await.unwrap();

    let pattern = get_stock_pattern(r.last().unwrap());

    RespBody::from(&pattern).response()
}
