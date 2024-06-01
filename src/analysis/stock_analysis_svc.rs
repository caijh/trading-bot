use std::error::Error;
use std::io::Cursor;

use polars::datatypes::DataType;
use polars::io::SerReader;
use polars::prelude::{col, IntoLazy, JsonReader};

use crate::analysis::stock_analysis_ctrl::Params;
use crate::analysis::stock_calculate::{down_at_least, ma};
use crate::analysis::stock_pattern::{get_stock_pattern, StockPattern};
use crate::index::stock_index::IndexConstituent;
use crate::index::stock_index_svc::{get_constituent_stocks, get_stock_index};
use crate::stock::stock_svc::get_stock_daily_price;

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
                    if down_at_least(prices, 3) {
                        focus_stocks.push(stock)
                    }
                }
                StockPattern::CrossStar => {
                    if down_at_least(prices, 3) {
                        focus_stocks.push(stock)
                    }
                }
                StockPattern::UnKnown => {
                    let json = serde_json::to_string(&prices)?;
                    let polars = JsonReader::new(Cursor::new(json)).finish();
                    let df = polars;
                    if let Ok(df) = df {
                        let df = df
                            .clone()
                            .lazy()
                            .select([col("close").cast(DataType::Float32)])
                            .collect()?;
                        let ma5 = ma(&df["close"], 5);
                        let ma20 = ma(&df["close"], 20);
                        let ma60 = ma(&df["close"], 60);
                        let pre_ma5 = ma5.get(ma5.len() - 2).unwrap();
                        let ma5 = ma5.last().unwrap();
                        let ma20 = ma20.last().unwrap();
                        let ma60 = ma60.last().unwrap();
                        if ma5 > pre_ma5
                            && ma5 >= ma20
                            && ma5 < ma60
                            && ((ma5 - ma20) / ma20 < 0.02)
                        {
                            focus_stocks.push(stock);
                        }
                    }
                }
            }
        }
    }
    Ok(focus_stocks)
}
