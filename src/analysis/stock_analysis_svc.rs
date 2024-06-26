use std::error::Error;

use crate::analysis::stock_analysis_ctrl::Params;
use crate::analysis::stock_analysis_model::AnalyzedStock;
use crate::analysis::stock_calculate::{down_at_least, max, mean, min};
use crate::analysis::stock_pattern::{get_stock_pattern, StockPattern};
use crate::index::stock_index_svc::{get_constituent_stocks, get_stock_index};
use crate::stock::stock_svc::get_stock_daily_price;

pub async fn analysis(params: &Params) -> Result<Vec<AnalyzedStock>, Box<dyn Error>> {
    let index = &params.index_code;
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
            StockPattern::BullishEngulfing | StockPattern::Piercing => {
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
