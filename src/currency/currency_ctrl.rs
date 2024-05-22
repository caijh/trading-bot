use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use web::response::RespBody;

use crate::currency::currency_svc;

pub fn currency_routers() -> Router {
    Router::new().route("/rate", get(get_rate))
}

pub async fn get_rate() -> impl IntoResponse {
    let r = currency_svc::get_rate().await;

    RespBody::from_result(&r).response()
}
