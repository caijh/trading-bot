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
use crate::stock::stock_model::{KLine, StockDailyPrice};

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
    /// 向上缺口
    UpGap,
    UpMA120,
    /// 未知形态
    UnKnown,
}

impl Display for StockPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StockPattern::LongLowerShadow => f.write_str("长下影线"),
            StockPattern::DojiStar => f.write_str("十字星"),
            StockPattern::Ma5Ma20 => f.write_str("Ma5>Ma20"),
            StockPattern::BullishEngulfing => f.write_str("看涨吞没形态"),
            StockPattern::Piercing => f.write_str("刺透形态"),
            StockPattern::UpGap => f.write_str("向上缺口"),
            StockPattern::UpMA120 => f.write_str("120均线"),
            StockPattern::UnKnown => f.write_str("未知形态"),
        }
    }
}

pub fn get_stock_pattern(prices: &[StockDailyPrice]) -> StockPattern {
    if prices.len() < 2 {
        return StockPattern::UnKnown;
    }

    let price = prices.last().unwrap();
    let open = &price.open;
    let close = &price.close;
    let factor_1 = BigDecimal::from_str("1.5").unwrap();
    let factor_2 = BigDecimal::from_str("2").unwrap();
    let real_body = price.get_real_body();
    let lower_shadow = price.get_lower_shadow();
    let upper_shadow = price.get_upper_shadow();
    if !price.is_down() {
        // 下影线长度是实体长度的2倍并且下影线长度要大于上影线长度
        if lower_shadow > real_body.clone() * factor_2.clone()
            && lower_shadow > upper_shadow.clone() * factor_1.clone()
        {
            return StockPattern::LongLowerShadow;
        }

        // 十字星
        let p: Decimal = open.clone() / close.clone();
        if p > Decimal::new("0.999").unwrap() && lower_shadow > upper_shadow {
            return StockPattern::DojiStar;
        }

        let pre_price = prices.get(prices.len() - 2);
        if let Some(pre_price) = pre_price {
            let pre_open = &pre_price.open;
            let pre_close = &pre_price.close;
            if pre_price.is_down() {
                let pre_real_body: BigDecimal = pre_price.get_real_body();
                if price.open < pre_close.clone()
                    && price.close > pre_open.clone()
                    && real_body > pre_real_body
                    && real_body > upper_shadow.clone() * factor_1.clone()
                {
                    // 看涨吞没
                    return StockPattern::BullishEngulfing;
                }

                let mid_price =
                    (pre_open.clone() + pre_close.clone()) / Decimal::from_str("2").unwrap();
                if price.is_up()
                    && price.open < pre_close.clone()
                    && close > &mid_price
                    && close < pre_open
                {
                    // 刺透
                    return StockPattern::Piercing;
                }

                if price.is_up() && price.open.clone() > pre_price.open.clone() {
                    // 向上缺口 
                    return StockPattern::UpGap;
                }
            }
        }
    } else {
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
        let close_df = df
            .clone()
            .lazy()
            .select([col("close").cast(DataType::Float32)])
            .collect()
            .unwrap();
        let ma5 = ma(&close_df["close"], 5);
        let ma20 = ma(&close_df["close"], 20);
        let ma60 = ma(&close_df["close"], 60);
        let ma120 = ma(&close_df["close"], 120);
        let pre_ma5 = ma5.get(ma5.len() - 2).unwrap();
        let ma5_last = ma5.last().unwrap();
        let ma20_last = ma20.last().unwrap();
        let ma60_last = ma60.last().unwrap();
        let ma120_last = ma120.last().unwrap();
        let ma120_last_pre = ma120.get(ma120.len() - 2).unwrap();

        // check volume
        let volume_df = df
            .clone()
            .lazy()
            .select([col("volume").cast(DataType::Float32)])
            .collect()
            .unwrap();
        let ma5_volume = ma(&volume_df["volume"], 5);
        let ma5_volume = ma5_volume.last().unwrap();
        let ma20_volume = ma(&volume_df["volume"], 20);
        let ma20_volume = ma20_volume.last().unwrap();

        if price.is_up()
            && ma5_last > pre_ma5
            && ma5_last >= ma20_last
            && ma5_last < ma60_last
            && ma5_volume >= ma20_volume
            && (real_body > upper_shadow.clone() * factor_1.clone())
            && (ma5 == ma20 || ((ma5_last - ma20_last) / ma20_last < 0.006))
        {
            return StockPattern::Ma5Ma20;
        }

        
        let pre_price = prices.get(prices.len() - 2).unwrap();
        if price.is_up() 
            && price.close > Decimal::from_f32(ma120_last.clone()).unwrap()
            && pre_price.close < Decimal::from_f32(ma120_last_pre.clone()).unwrap()
        {
            return StockPattern::UpMA120;
        }
    }

    StockPattern::UnKnown
}
