use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use web::response::RespBody;

use crate::debt_svc;

#[derive(Serialize, Deserialize, Clone)]
pub struct DebtPrice {
    pub current: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub zd: String,
    pub zdf: String,
    pub yc: String,
    pub v: String,
    pub cje: String,
    pub t: String,
}

pub fn debt_routers() -> Router {
    Router::new().route("/price", get(get_debt_price))
}

#[derive(Serialize, Deserialize)]
struct Params {
    code: String,
}

async fn get_debt_price(Query(params): Query<Params>) -> impl IntoResponse {
    let r = debt_svc::get_debt_price(&params.code).await;
    RespBody::from_result(&r).response()
}
