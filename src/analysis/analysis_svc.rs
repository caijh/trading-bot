use crate::analysis::analysis_ctrl::{IndexAnalysisParams, StockAnalysisParams};
use crate::analysis::analysis_model::AnalyzedStock;
use crate::analysis::stock_calculate::first_max_min;
use crate::analysis::stock_pattern::get_candlestick_patterns;
use crate::analysis::stock_pattern::get_ma_patterns;
use crate::fund::fund_svc;
use crate::index::index_svc::{get_constituent_stocks, get_stock_index};
use crate::stock::stock_svc;
use crate::stock::stock_svc::get_stock_daily_price;
use polars::io::SerReader;
use polars::prelude::JsonReader;
use std::error::Error;
use std::io::Cursor;
use std::ops::Not;
use tracing::{error, info};

pub async fn analysis_index(
    params: &IndexAnalysisParams,
) -> Result<Vec<AnalyzedStock>, Box<dyn Error>> {
    let index = &params.code.clone().unwrap();
    let index = get_stock_index(index).await?;
    let stocks = get_constituent_stocks(&index.code).await?;
    let mut focus_stocks: Vec<AnalyzedStock> = Vec::new();
    for stock in stocks {
        let params = StockAnalysisParams {
            code: stock.stock_code.clone(),
        };
        let r = analysis_stock(&params).await?;
        if let Some(item) = r {
            if item.pattern.len() >= 2 {
                focus_stocks.push(item);
            }
        }
    }
    Ok(focus_stocks)
}

pub async fn analysis_stock(
    params: &StockAnalysisParams,
) -> Result<Option<AnalyzedStock>, Box<dyn Error>> {
    let stock = stock_svc::get_stock(&params.code).await?;
    let prices = get_stock_daily_price(&stock.code).await?;

    if prices.len() < 2 {
        return Ok(None);
    }

    let json = serde_json::to_string(&prices).unwrap();
    let polars = JsonReader::new(Cursor::new(json)).finish();
    let df: polars::prelude::DataFrame = polars?;
    let candlestick_patterns = get_candlestick_patterns();
    let ma_patterns = get_ma_patterns();
    let mut match_patterns = Vec::new();
    for pattern in candlestick_patterns {
        if pattern.is_match(&stock, &prices, &df) {
            info!("pattern {} matched", pattern.name());
            match_patterns.push(pattern.name());
        }
    }
    if match_patterns.is_empty().not() {
        let match_ma_patterns = ma_patterns
            .iter()
            .filter(|ma| ma.is_match(&stock, &prices, &df))
            .collect::<Vec<_>>();
        if !match_ma_patterns.is_empty() {
            for ele in match_ma_patterns {
                info!("pattern {} matched", ele.name());
                match_patterns.push(ele.name());
            }
        }
    }

    let (max, min) = first_max_min(&df, &prices);
    let current = prices.last().unwrap().close.clone();
    let analyzed_stock = Some(AnalyzedStock {
        code: stock.code.to_string(),
        name: stock.name.to_string(),
        current,
        min,
        max,
        pattern: match_patterns,
    });
    Ok(analyzed_stock)
}

pub async fn analysis_funds(code: Option<String>) -> Result<Vec<AnalyzedStock>, Box<dyn Error>> {
    let funds = fund_svc::find_all().await?;
    let mut focus_stocks: Vec<AnalyzedStock> = Vec::new();
    if funds.is_empty() {
        return Ok(focus_stocks);
    }

    // filter funds by code if provided
    let funds = match code {
        None => funds,
        Some(code) => funds.into_iter().filter(|f| f.code == code).collect(),
    };

    for fund in funds {
        let params = StockAnalysisParams {
            code: fund.code.clone(),
        };
        let r = analysis_stock(&params).await;
        match r {
            Ok(r) => {
                if let Some(item) = r {
                    if item.pattern.len() >= 2 {
                        focus_stocks.push(item);
                    }
                }
            }
            Err(e) => {
                error!("analysis_stock {} fail, {}", &params.code, e)
            }
        }
    }
    Ok(focus_stocks)
}
