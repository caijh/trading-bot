use std::str::FromStr;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use web::response::RespBody;
use crate::exchange::Exchange;

use crate::stock_svc::{get_stock_daily_price, get_stock_price, sync_stocks};

pub fn stock_routers() -> Router {
    Router::new()
        .route("/sync/:exchange", get(sync))
        .route("/:code/daily", get(stock_daily))
        .route("/:code/price", get(stock_price))
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
pub async fn sync(Path(exchange): Path<String>) -> impl IntoResponse {
    let exchange = Exchange::from_str(&exchange).unwrap();
    
    let r = sync_stocks(&exchange).await;

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
