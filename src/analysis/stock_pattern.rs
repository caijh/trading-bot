use std::fmt::Display;
use std::fmt::Formatter;
use std::io::Cursor;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use polars::datatypes::DataType;
use polars::io::SerReader;
use polars::prelude::{col, IntoLazy, JsonReader};
use rbatis::rbdc::Decimal;
use serde::{Deserialize, Serialize};

use crate::analysis::stock_calculate::ma;
use crate::stock::stock_model::StockDailyPrice;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum StockPattern {
    /// 长下影线
    LongLowerShadow,
    /// 十字星
    DojiStar,
    /// MA5 > MA20
    Ma5Ma20,
    /// 看涨吞没形态
    BullishEngulfing,
    /// 刺透形态
    Piercing,
    /// 未知形态
    UnKnown,
}

impl Display for StockPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StockPattern::LongLowerShadow => f.write_str("长下影线"),
            StockPattern::DojiStar => f.write_str("十字星"),
            StockPattern::Ma5Ma20 => f.write_str("Ma5>Ma20"),
            StockPattern::UnKnown => f.write_str("Unknown"),
            StockPattern::BullishEngulfing => f.write_str("看涨吞没形态"),
            StockPattern::Piercing => f.write_str("刺透形态"),
        }
    }
}

pub fn get_stock_pattern(prices: &[StockDailyPrice]) -> StockPattern {
    let price = prices.last().unwrap();
    let open = &price.open;
    let close = &price.close;
    let low = &price.low;
    let high = &price.high;
    let factor_1 = BigDecimal::from_str("1.5").unwrap();
    let factor_2 = BigDecimal::from_str("2").unwrap();
    let lower_shadow: BigDecimal;
    let upper_shadow: BigDecimal;
    let real_body: BigDecimal = (open.clone() - close.clone()).abs();
    if open <= close {
        lower_shadow = (low.clone() - open.clone()).abs();
        upper_shadow = (close.clone() - high.clone()).abs();

        // 下影线长度是实体长度的2倍并且下影线长度要大于上影线长度
        if lower_shadow > real_body.clone() * factor_2.clone()
            && lower_shadow > upper_shadow.clone() * factor_1.clone()
        {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = open.clone() / close.clone();
        if p > Decimal::new("0.999").unwrap() && lower_shadow > upper_shadow {
            return StockPattern::DojiStar;
        }

        let pre_price = prices.get(prices.len() - 2);
        if let Some(pre_price) = pre_price {
            let pre_open = &pre_price.open;
            let pre_close = &pre_price.close;
            if pre_open > pre_close {
                let pre_real_body: BigDecimal = (open.clone() - close.clone()).abs();
                if pre_real_body < real_body {
                    return StockPattern::BullishEngulfing;
                }
            }
            if pre_open < pre_close {
                let mid_price =
                    (pre_open.clone() + pre_close.clone()) / Decimal::from_str("2").unwrap();
                if open < close && close > &mid_price {
                    return StockPattern::Piercing;
                }
            }
        }
    } else {
        lower_shadow = (low.clone() - close.clone()).abs();
        upper_shadow = (open.clone() - high.clone()).abs();

        // 下影线长度是实体长度的2倍并且下影线长度要大于上影线长度
        if lower_shadow > real_body.clone() * factor_2.clone()
            && lower_shadow > upper_shadow.clone() * factor_1.clone()
        {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = close.clone() / open.clone();
        if p > Decimal::new("0.999").unwrap() && lower_shadow > upper_shadow {
            return StockPattern::DojiStar;
        }
    }

    let json = serde_json::to_string(&prices).unwrap();
    let polars = JsonReader::new(Cursor::new(json)).finish();
    let df = polars;
    if let Ok(df) = df {
        let df = df
            .clone()
            .lazy()
            .select([col("close").cast(DataType::Float32)])
            .collect()
            .unwrap();
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
            && ((ma5 - ma20) / ma20 < 0.01)
            && (real_body > upper_shadow * factor_1.clone())
        {
            return StockPattern::Ma5Ma20;
        }
    }

    StockPattern::UnKnown
}
