use axum::response::IntoResponse;
use web::response::RespBody;

use crate::stock::init_stocks;

pub async fn init() -> impl IntoResponse {
    let r = init_stocks().await;
    RespBody::from_result(&r).response()
}