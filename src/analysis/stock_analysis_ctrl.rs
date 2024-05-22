use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use web::response::RespBody;

use crate::analysis::stock_analysis_svc;

pub fn stock_analysis_routers() -> Router {
    Router::new().route("/", get(analysis))
}

#[derive(Serialize, Deserialize)]
pub struct Params {
    pub index_code: String,
}

pub async fn analysis(Query(params): Query<Params>) -> impl IntoResponse {
    let r = stock_analysis_svc::analysis(&params).await;

    RespBody::from_result(&r).response()
}
