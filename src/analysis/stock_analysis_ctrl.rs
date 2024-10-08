use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Query;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::analysis::stock_analysis_svc;

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
