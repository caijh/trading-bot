use application_web::response::RespBody;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::analysis::stock_analysis_svc;

pub fn stock_analysis_routers() -> Router {
    Router::new()
        .route("/index", get(analysis_index))
        .route("/stock", get(analysis_stock))
        .route("/funds", get(analysis_funds))
}

#[derive(Serialize, Deserialize)]
pub struct IndexAnalysisParams {
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct StockAnalysisParams {
    pub code: String,
}

async fn analysis_index(Query(params): Query<IndexAnalysisParams>) -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_index(&params).await;

    RespBody::from_result(&r).response()
}

async fn analysis_stock(Query(params): Query<StockAnalysisParams>) -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_stock(&params).await;

    RespBody::from_result(&r).response()
}

async fn analysis_funds() -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_funds().await;

    RespBody::from_result(&r).response()
}
