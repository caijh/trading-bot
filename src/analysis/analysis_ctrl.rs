use application_core::lang::runnable::Runnable;
use application_web::response::RespBody;
use application_web_macros::get;
use axum::extract::Query;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio::spawn;

use crate::analysis::analysis_svc;
use crate::job::jobs::{AnalysisFundsJob, AnalysisIndexStocksJob};

#[derive(Serialize, Deserialize)]
pub struct IndexAnalysisParams {
    pub code: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FundsAnalysisParams {
    pub code: Option<String>,
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
    let code = params.code.clone();
    spawn(async {
        let job = AnalysisIndexStocksJob { code };

        job.run().await;
    });

    RespBody::<()>::success_info("Analysis index Stocks in background")
}

/// 获取基金分析结果
///
/// # Returns
/// * `impl IntoResponse` - 返回一个实现了`IntoResponse` trait的响应对象，包含基金分析数据
#[get("/analysis/funds")]
async fn analysis_funds(Query(params): Query<FundsAnalysisParams>) -> impl IntoResponse {
    let code = params.code.clone();
    spawn(async {
        let job = AnalysisFundsJob { code };

        job.run().await;
    });

    RespBody::<()>::success_info("Analysis fund Stocks in background")
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
    let r = analysis_svc::analysis_stock(&params).await;

    RespBody::result(&r).response()
}
