use std::error::Error;

use context::SERVICES;
use database::DbService;

use crate::analysis::stock_analysis_ctrl::{IndexAnalysisParams, StockAnalysisParams};
use crate::analysis::stock_analysis_model::AnalyzedStock;
use crate::analysis::stock_calculate::{down_at_least, max, mean, min};
use crate::analysis::stock_pattern::{get_stock_pattern, StockPattern};
use crate::index::stock_index_svc::{get_constituent_stocks, get_stock_index};
use crate::stock::stock_model::Stock;
use crate::stock::stock_svc::get_stock_daily_price;

pub async fn analysis_index(
    params: &IndexAnalysisParams,
) -> Result<Vec<AnalyzedStock>, Box<dyn Error>> {
    let index = &params.code;
    let index = get_stock_index(index).await?;
    let stocks = get_constituent_stocks(&index.code).await?;
    let mut focus_stocks: Vec<AnalyzedStock> = Vec::new();
    for stock in stocks {
        let prices = get_stock_daily_price(&stock.stock_code).await?;
        let pattern = get_stock_pattern(&prices);
        let max = max(&prices, 20);
        let min = min(&prices, 20);
        let current = prices.last().unwrap().close.clone();
        let mean = mean(&prices, 120);
        match pattern {
            StockPattern::UnKnown => {}
            StockPattern::LongLowerShadow | StockPattern::DojiStar => {
                if down_at_least(&prices, 4) && current > mean {
                    focus_stocks.push(AnalyzedStock {
                        code: stock.stock_code.to_string(),
                        name: stock.stock_name.to_string(),
                        pattern,
                        min,
                        max,
                        current,
                    });
                }
            }
            StockPattern::Ma5Ma20 => {
                if current > mean {
                    focus_stocks.push(AnalyzedStock {
                        code: stock.stock_code.to_string(),
                        name: stock.stock_name.to_string(),
                        pattern,
                        min,
                        max,
                        current,
                    });
                }
            }
            StockPattern::BullishEngulfing | StockPattern::Piercing | StockPattern::UpGap => {
                if down_at_least(&prices[0..prices.len() - 1], 3) && current > mean {
                    focus_stocks.push(AnalyzedStock {
                        code: stock.stock_code.to_string(),
                        name: stock.stock_name.to_string(),
                        pattern,
                        min,
                        max,
                        current,
                    });
                }
            }
        }
    }
    Ok(focus_stocks)
}

pub async fn analysis_stock(
    params: &StockAnalysisParams,
) -> Result<Option<AnalyzedStock>, Box<dyn Error>> {
    let dao = SERVICES.get::<DbService>().dao();
    let stock = Stock::select_by_code(dao, &params.code).await?;
    if stock.is_none() {
        return Err("Stock not found".into());
    }

    let stock = stock.unwrap();
    let prices = get_stock_daily_price(&stock.code).await?;
    let pattern = get_stock_pattern(&prices);
    let max = max(&prices, 20);
    let min = min(&prices, 20);
    let current = prices.last().unwrap().close.clone();
    let mean = mean(&prices, 120);
    let mut analyzed_stock = Option::None;
    match pattern {
        StockPattern::UnKnown => {}
        StockPattern::LongLowerShadow | StockPattern::DojiStar => {
            if down_at_least(&prices, 4) && current > mean {
                analyzed_stock = Some(AnalyzedStock {
                    code: stock.code.to_string(),
                    name: stock.name.to_string(),
                    pattern,
                    min,
                    max,
                    current,
                });
            }
        }
        StockPattern::Ma5Ma20 => {
            if current > mean {
                analyzed_stock = Some(AnalyzedStock {
                    code: stock.code.to_string(),
                    name: stock.name.to_string(),
                    pattern,
                    min,
                    max,
                    current,
                });
            }
        }
        StockPattern::BullishEngulfing | StockPattern::Piercing | StockPattern::UpGap => {
            if down_at_least(&prices[0..prices.len() - 1], 3) && current > mean {
                analyzed_stock = Some(AnalyzedStock {
                    code: stock.code.to_string(),
                    name: stock.name.to_string(),
                    pattern,
                    min,
                    max,
                    current,
                });
            }
        }
    }
    Ok(analyzed_stock)
}
