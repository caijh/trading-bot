use application_core::lang::runnable::Runnable;
use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Query;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::spawn;

use crate::analysis::stock_analysis_svc;
use crate::job::jobs::{AnalysisFundsJob, AnalysisIndexStocksJob};

#[derive(Serialize, Deserialize)]
pub struct IndexAnalysisParams {
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct StockAnalysisParams {
    pub code: String,
}

#[get("/analysis/index")]
async fn analysis_index(Query(params): Query<IndexAnalysisParams>) -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_index(&params).await;

    RespBody::result(&r).response()
}

#[get("/analysis/index/all")]
async fn analysis_index_all() -> impl IntoResponse {
    spawn(async {
        let job = AnalysisIndexStocksJob;

        job.run().await;
    });

    RespBody::<()>::success_info("Analysis index Stocks in background")
}

#[get("/analysis/stock")]
async fn analysis_stock(Query(params): Query<StockAnalysisParams>) -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_stock(&params).await;

    RespBody::result(&r).response()
}

#[get("/analysis/funds")]
async fn analysis_funds() -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_funds().await;

    RespBody::result(&r).response()
}

#[get("/analysis/funds/all")]
async fn analysis_funds_all() -> impl IntoResponse {
    spawn(async {
        let job = AnalysisFundsJob;

        job.run().await;
    });

    RespBody::<()>::success_info("Analysis fund Stocks in background")
}
