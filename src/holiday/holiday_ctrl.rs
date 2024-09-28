use application_web::response::RespBody;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use chrono::Local;

use crate::holiday::holiday_svc::{is_holiday, sync_holidays};

pub fn holiday_routers() -> Router {
    Router::new()
        .route("/sync", get(sync))
        .route("/today", get(today_is_holiday))
}

/// `today_is_holiday`是一个公共异步函数
///
/// 此函数检查今天是否为假日。如果今天是假日，返回`true`；否则，返回`false`.
///
/// 这个函数首先获取当前的本地时间，然后用`is_holiday`函数来检查这个日期是否为假日。
///
/// 最后，将结果封装在`RespBody`中，并作为响应返回。
async fn today_is_holiday() -> impl IntoResponse {
    let now = Local::now();
    let r = is_holiday(&now).await;

    RespBody::from_result(&r).response()
}

/// 定义一个异步函数sync，返回类型为IntoResponse的实现
/// 该函数首先调用sync_holidays异步方法获取数据
/// 然后将结果转换为RespBody，并构建一个响应对象
async fn sync() -> impl IntoResponse {
    let r = sync_holidays().await;

    RespBody::from_result(&r).response()
}
