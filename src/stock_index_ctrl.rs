use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use web::response::RespBody;

use crate::stock_index_svc;

pub fn stock_index_routers() -> Router {
    Router::new()
        .route("/:code/sync", get(sync))
        .route("/:code/stocks", get(get_stocks))
}

pub async fn get_stocks(Path(code): Path<String>) -> impl IntoResponse {
    let r = stock_index_svc::get_constituent_stocks(&code).await;

    RespBody::from_result(&r).response()
}

pub async fn sync(Path(code): Path<String>) -> impl IntoResponse {
    let r = stock_index_svc::sync_constituents(&code).await;

    RespBody::from_result(&r).response()
}
