
use anyhow::Ok;
use application_core::lang::runnable::Runnable;
use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Path;
use axum::response::IntoResponse;
use tokio::spawn;

use crate::index::stock_index_svc;
use crate::job::jobs::{SyncIndexStocksJob, SyncAllIndexStockPriceJob};

#[get("/index/:code/stocks")]
pub async fn get_stocks(Path(code): Path<String>) -> impl IntoResponse {
    let r = stock_index_svc::get_constituent_stocks(&code).await;

    RespBody::result(&r).response()
}

#[get("/index/sync/:code")]
pub async fn sync(Path(code): Path<String>) -> impl IntoResponse {
    let r = stock_index_svc::sync_constituents(&code).await;

    RespBody::result(&r).response()
}

#[get("/index/sync/all")]
pub async fn sync_all() -> impl IntoResponse {
    spawn(async {
        let job = SyncIndexStocksJob;

        job.run().await;

        Ok(())
    });

    RespBody::<()>::success_info("Sync index Stocks in background")
}

/// 同步所有指数中股票的价格
#[get("/index/sync/all/price")]
pub async fn sync_index_stock_price() -> impl IntoResponse {
    spawn(async {
        let job = SyncAllIndexStockPriceJob;

        job.run().await;

        Ok(())
    });

    RespBody::<()>::success_info("Sync index Stocks prices in background")
}
