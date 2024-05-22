use std::error::Error;

use crate::calculate;
use crate::index::stock_index::IndexConstituent;
use crate::index::stock_index_svc::{get_constituent_stocks, get_stock_index};
use crate::stock_analysis_ctrl::Params;
use crate::stock_pattern::{get_stock_pattern, StockPattern};
use crate::stock_svc::get_stock_daily_price;

pub async fn analysis(params: &Params) -> Result<Vec<IndexConstituent>, Box<dyn Error>> {
    let index = &params.index_code;
    let index = get_stock_index(index).await?;
    let stocks = get_constituent_stocks(&index.code).await?;
    let mut focus_stocks = Vec::new();
    for stock in stocks {
        let prices = get_stock_daily_price(&stock.stock_code).await?;
        let l = prices.last();
        if let Some(price) = l {
            let pattern = get_stock_pattern(price);
            match pattern {
                StockPattern::LongLowerShadow => {
                    if calculate::down_at_least(prices, 3) {
                        focus_stocks.push(stock)
                    }
                }
                StockPattern::CrossStar => {
                    if calculate::down_at_least(prices, 3) {
                        focus_stocks.push(stock)
                    }
                }
                StockPattern::UnKnown => {}
            }
        }
    }
    Ok(focus_stocks)
}
