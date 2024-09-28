use application_web::response::RespBody;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::analysis::stock_pattern::get_stock_pattern;
use crate::stock::stock_svc;
use crate::stock::stock_svc::{get_stock_daily_price, get_stock_price};

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
    let r = stock_svc::sync(&exchange).await;
    RespBody::from_result(&r).response()
}

/// 获取股票日线价格
async fn stock_daily(Query(params): Query<StockParams>) -> impl IntoResponse {
    let r = get_stock_daily_price(&params.code).await;
    RespBody::from_result(&r).response()
}

/// 获取股票当前价格
async fn stock_price(Query(params): Query<StockParams>) -> impl IntoResponse {
    let r = get_stock_price(&params.code).await;
    RespBody::from_result(&r).response()
}

async fn stock_pattern(Query(params): Query<StockParams>) -> impl IntoResponse {
    let prices = get_stock_daily_price(&params.code).await.unwrap();

    let pattern = get_stock_pattern(&prices);

    RespBody::from(&pattern).response()
}
