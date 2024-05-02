use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use chrono::Local;
use web::response::RespBody;

use crate::holiday_svc::{is_holiday, sync_holidays};

pub fn holiday_routers() -> Router {
    Router::new()
        .route("/syc", get(sync_holiday))
        .route("/today", get(today_is_holiday))
}

pub async fn today_is_holiday() -> impl IntoResponse {
    let now = Local::now();
    let r = is_holiday(&now).await;

    RespBody::from_result(&r).response()
}

pub async fn sync_holiday() -> impl IntoResponse {
    let r = sync_holidays().await;

    RespBody::from_result(&r).response()
}
