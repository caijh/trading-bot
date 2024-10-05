use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Path;
use axum::response::IntoResponse;

use crate::index::stock_index_svc;

#[get("/index/:code/stocks")]
pub async fn get_stocks(Path(code): Path<String>) -> impl IntoResponse {
    let r = stock_index_svc::get_constituent_stocks(&code).await;

    RespBody::from_result(&r).response()
}

#[get("/index/sync/:code")]
pub async fn sync(Path(code): Path<String>) -> impl IntoResponse {
    let r = stock_index_svc::sync_constituents(&code).await;

    RespBody::from_result(&r).response()
}
