use application_core::lang::runnable::Runnable;
use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Query;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::spawn;

use crate::analysis::stock_analysis_svc;
use crate::job::jobs::{AnalysisFundsJob, AnalysisIndexStocksJob};

#[derive(Serialize, Deserialize)]
pub struct IndexAnalysisParams {
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct StockAnalysisParams {
    pub code: String,
}

/// 分析指数中的股票
///
/// # Parameters
///
/// * `Query(params)` - 从请求中提取的查询参数，用于指数分析。
///
/// # Returns
///
/// 返回一个实现了`IntoResponse`的类型，用于生成HTTP响应。
#[get("/analysis/index")]
async fn analysis_index(Query(params): Query<IndexAnalysisParams>) -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_index(&params).await;

    RespBody::result(&r).response()
}

/// 在后台分析所有指数中的股票
///
/// 该函数通过异步任务启动一个分析股票指数的作业，而不阻塞主线程
#[get("/analysis/index/all")]
async fn analysis_index_all() -> impl IntoResponse {
    spawn(async {
        let job = AnalysisIndexStocksJob;

        job.run().await;
    });

    RespBody::<()>::success_info("Analysis index Stocks in background")
}

/// 调用股票分析服务进行处理，并返回分析结果。
///
/// # 参数
///
/// - `Query(params)` - 一个从查询字符串中解析出来的`StockAnalysisParams`对象，包含进行股票分析所需的各种参数。
///
/// # 返回值
///
/// 返回一个实现了`IntoResponse` trait的对象，通常是一个HTTP响应。响应体中包含股票分析的结果。
/// 如果分析过程中遇到错误，将返回一个表示错误的HTTP响应。
#[get("/analysis/stock")]
async fn analysis_stock(Query(params): Query<StockAnalysisParams>) -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_stock(&params).await;

    RespBody::result(&r).response()
}

/// 获取基金分析结果
///
/// # Returns
/// * `impl IntoResponse` - 返回一个实现了`IntoResponse` trait的响应对象，包含基金分析数据
#[get("/analysis/funds")]
async fn analysis_funds() -> impl IntoResponse {
    let r = stock_analysis_svc::analysis_funds().await;

    RespBody::result(&r).response()
}

/// 在后台执行基金分析任务
///
/// 本函数通过异步方式启动一个基金分析任务，该任务在后台运行，不影响当前请求的响应
/// 主要用途是当用户请求全面的基金分析时，立即返回响应信息，避免用户等待长时间的分析过程完成
#[get("/analysis/funds/all")]
async fn analysis_funds_all() -> impl IntoResponse {
    // 异步执行基金分析任务
    spawn(async {
        let job = AnalysisFundsJob;

        job.run().await;
    });

    RespBody::<()>::success_info("Analysis fund Stocks in background")
}
