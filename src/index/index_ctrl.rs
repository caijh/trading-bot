use application_core::lang::runnable::Runnable;
use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Path;
use axum::response::IntoResponse;
use tokio::spawn;

use crate::index::index_svc;
use crate::job::jobs::{SyncAllIndexStockPriceJob, SyncIndexStocksJob};

#[get("/index/:code/stocks")]
pub async fn get_stocks(Path(code): Path<String>) -> impl IntoResponse {
    let r = index_svc::get_constituent_stocks(&code).await;

    RespBody::result(&r).response()
}

/// 后台同步指定指数的股票信息
#[get("/index/sync/:code")]
pub async fn sync(Path(code): Path<String>) -> impl IntoResponse {
    let r = index_svc::sync_constituents(&code).await;

    RespBody::result(&r).response()
}

/// 后台所有指数的股票信息
///
/// 该函数通过异步任务启动一个后台作业，用于同步指数股票信息，
/// 并立即返回成功信息给前端，不会等待同步任务完成。
///
/// # Returns
///
/// 返回一个实现IntoResponse的类型，通常是一个HTTP响应，
/// 表示后台同步任务已成功启动。
#[get("/index/sync")]
pub async fn sync_all() -> impl IntoResponse {
    spawn(async {
        let job = SyncIndexStocksJob;
        job.run().await;
    });

    RespBody::<()>::success_info("Sync index Stocks in background")
}

/// 同步所有指数中股票的价格
#[get("/index/sync/price")]
pub async fn sync_index_stock_price() -> impl IntoResponse {
    spawn(async {
        let job = SyncAllIndexStockPriceJob;

        job.run().await;
    });

    RespBody::<()>::success_info("Sync index Stocks prices in background")
}
