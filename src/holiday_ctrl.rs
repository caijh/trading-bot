use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use chrono::Local;
use web::response::RespBody;
use crate::holiday_svc::is_holiday;

pub fn holiday_routers() -> Router {
    Router::new()
        .route("/today", get(today_is_holiday))
}

pub async fn today_is_holiday() -> impl IntoResponse {
    let now = Local::now();
    let r = is_holiday(&now).await;

    RespBody::from_result(&r).response()
}
