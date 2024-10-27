use crate::analysis::stock_pattern::get_stock_pattern;
use crate::job::jobs::SyncStocksJob;
use crate::stock::stock_svc::{get_stock_daily_price, get_stock_price};
use anyhow::Ok;
use application_core::lang::runnable::Runnable;
use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::spawn;

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
#[get("/stock/sync/:exchange")]
async fn sync(Path(exchange): Path<String>) -> impl IntoResponse {
    spawn(async {
        let job = SyncStocksJob { exchange };
        job.run().await;
        Ok(())
    });

    RespBody::<()>::success_info("Sync Stocks in backgroud")
}

/// 获取股票日线价格
#[get("/stock/daily")]
async fn stock_daily(Query(params): Query<StockParams>) -> impl IntoResponse {
    let r = get_stock_daily_price(&params.code).await;
    RespBody::result(&r).response()
}

/// 获取股票当前价格
#[get("/stock/price")]
async fn stock_price(Query(params): Query<StockParams>) -> impl IntoResponse {
    let r = get_stock_price(&params.code).await;
    RespBody::result(&r).response()
}

#[get("/stock/pattern")]
async fn stock_pattern(Query(params): Query<StockParams>) -> impl IntoResponse {
    let prices = get_stock_daily_price(&params.code).await.unwrap();

    let pattern = get_stock_pattern(&prices);

    RespBody::success(&pattern).response()
}
