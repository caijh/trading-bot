use application_web::response::RespBody;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::debt::debt_svc;

#[derive(Serialize, Deserialize)]
struct Params {
    code: String,
}

pub fn debt_routers() -> Router {
    Router::new().route("/price", get(get_debt_price))
}

async fn get_debt_price(Query(params): Query<Params>) -> impl IntoResponse {
    let r = debt_svc::get_debt_price(&params.code).await;
    RespBody::from_result(&r).response()
}
