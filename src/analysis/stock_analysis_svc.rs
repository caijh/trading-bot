use crate::analysis::stock_analysis_ctrl::{IndexAnalysisParams, StockAnalysisParams};
use crate::analysis::stock_analysis_model::AnalyzedStock;
use crate::analysis::stock_calculate::{down_at_least, max, mean, min};
use crate::analysis::stock_pattern::{get_stock_pattern, StockPattern};
use crate::fund::fund_model::Fund;
use crate::index::stock_index_svc::{get_constituent_stocks, get_stock_index};
use crate::stock::stock_model::Stock;
use crate::stock::stock_svc::get_stock_daily_price;
use application_beans::factory::bean_factory::BeanFactory;
use application_context::context::application_context::APPLICATION_CONTEXT;
use database::DbService;
use std::error::Error;
use tracing::info;

const DOWN_AT_LEAST_DAYS: i32 = 4;

pub async fn analysis_index(
    params: &IndexAnalysisParams,
) -> Result<Vec<AnalyzedStock>, Box<dyn Error>> {
    let index = &params.code;
    let index = get_stock_index(index).await?;
    let stocks = get_constituent_stocks(&index.code).await?;
    let mut focus_stocks: Vec<AnalyzedStock> = Vec::new();
    for stock in stocks {
        let prices = get_stock_daily_price(&stock.stock_code).await;
        match prices {
            Ok(prices) => {
                let pattern = get_stock_pattern(&prices);
                let max = max(&prices, 20);
                let min = min(&prices, 20);
                let current = prices.last().unwrap().close.clone();
                let mean = mean(&prices, 120);
                match pattern {
                    StockPattern::UnKnown => {}
                    StockPattern::LongLowerShadow | StockPattern::DojiStar => {
                        if down_at_least(&prices, DOWN_AT_LEAST_DAYS) && current > mean {
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
                    StockPattern::BullishEngulfing
                    | StockPattern::Piercing
                    | StockPattern::UpGap => {
                        if down_at_least(&prices[0..prices.len() - 1], DOWN_AT_LEAST_DAYS - 1)
                            && current > mean
                        {
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
            Err(e) => {
                info!("fail to get stock daily price {:?}", e);
            }
        }
    }
    Ok(focus_stocks)
}

pub async fn analysis_stock(
    params: &StockAnalysisParams,
) -> Result<Option<AnalyzedStock>, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context
        .get_bean_factory()
        .get::<DbService>()
        .dao();
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
    if current > mean {
        // 当前价大于120天均价
        match pattern {
            StockPattern::UnKnown => {}
            StockPattern::LongLowerShadow | StockPattern::DojiStar => {
                // 长下影、十字星
                if down_at_least(&prices, DOWN_AT_LEAST_DAYS) {
                    // 连续下跌
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
                // MA5 > MA20
                analyzed_stock = Some(AnalyzedStock {
                    code: stock.code.to_string(),
                    name: stock.name.to_string(),
                    pattern,
                    min,
                    max,
                    current,
                });
            }
            StockPattern::BullishEngulfing | StockPattern::Piercing | StockPattern::UpGap => {
                // 看涨吞没形态、刺透形态、向上缺口
                if down_at_least(&prices[0..prices.len() - 1], DOWN_AT_LEAST_DAYS - 1) {
                    // 之前连续下跌3天
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
    }
    Ok(analyzed_stock)
}

pub async fn analysis_funds() -> Result<Vec<AnalyzedStock>, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context
        .get_bean_factory()
        .get::<DbService>()
        .dao();
    let funds = Fund::select_all(dao).await?;
    let mut focus_stocks: Vec<AnalyzedStock> = Vec::new();
    if funds.is_empty() {
        return Ok(focus_stocks);
    }

    for fund in funds {
        let prices = get_stock_daily_price(&fund.code).await?;
        let pattern = get_stock_pattern(&prices);
        let max = max(&prices, 20);
        let min = min(&prices, 20);
        let current = prices.last().unwrap().close.clone();
        let mean = mean(&prices, 120);
        if current > mean {
            // 当前价大于120天均价
            match pattern {
                StockPattern::UnKnown => {}
                StockPattern::LongLowerShadow | StockPattern::DojiStar => {
                    // 长下影、十字星
                    if down_at_least(&prices, DOWN_AT_LEAST_DAYS) {
                        // 连续下跌
                        focus_stocks.push(AnalyzedStock {
                            code: fund.code.to_string(),
                            name: fund.name.to_string(),
                            pattern,
                            min,
                            max,
                            current,
                        });
                    }
                }
                StockPattern::Ma5Ma20 => {
                    // MA5 > MA20
                    focus_stocks.push(AnalyzedStock {
                        code: fund.code.to_string(),
                        name: fund.name.to_string(),
                        pattern,
                        min,
                        max,
                        current,
                    });
                }
                StockPattern::BullishEngulfing | StockPattern::Piercing | StockPattern::UpGap => {
                    // 看涨吞没形态、刺透形态、向上缺口
                    if down_at_least(&prices[0..prices.len() - 1], DOWN_AT_LEAST_DAYS) {
                        // 之前连续下跌3天
                        focus_stocks.push(AnalyzedStock {
                            code: fund.code.to_string(),
                            name: fund.name.to_string(),
                            pattern,
                            min,
                            max,
                            current,
                        });
                    }
                }
            }
        }
    }
    Ok(focus_stocks)
}
