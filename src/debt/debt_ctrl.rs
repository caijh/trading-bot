use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Query;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::debt::debt_svc;

#[derive(Serialize, Deserialize)]
struct Params {
    code: String,
}

#[get("/debt/price")]
async fn get_debt_price(Query(params): Query<Params>) -> impl IntoResponse {
    let r = debt_svc::get_debt_price(&params.code).await;
    RespBody::from_result(&r).response()
}
