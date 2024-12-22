use std::str::FromStr;

use crate::analysis::stock_calculate::{down_at_least, ma};
use crate::stock::stock_model;
use crate::stock::stock_price_model::{KLine, Model as StockDailyPrice};
use bigdecimal::{BigDecimal, FromPrimitive};
use polars::datatypes::DataType;
use polars::frame::DataFrame;
use polars::prelude::{col, IntoLazy};
use serde::{Deserialize, Serialize};

const DOWN_AT_LEAST_DAYS: i32 = 3;

pub trait StockPattern {
    fn is_match(
        &self,
        stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        df: &DataFrame,
    ) -> bool;

    fn name(&self) -> String;
}

/// 锤子线
#[derive(Serialize, Deserialize, Clone)]
pub struct HammerPattern {}

impl StockPattern for HammerPattern {
    fn is_match(
        &self,
        _stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        _df: &DataFrame,
    ) -> bool {
        let _price = prices.last().unwrap();
        let pre_price = prices.get(prices.len() - 2).unwrap();
        let price = prices.last().unwrap();
        let factor_1 = BigDecimal::from_str("4").unwrap();
        let factor_2 = BigDecimal::from_str("2").unwrap();
        let real_body = price.get_real_body();
        let lower_shadow = price.get_lower_shadow();
        let upper_shadow = price.get_upper_shadow();
        // 下影线长度是实体长度的2倍并且下影线长度要大于上影线长度
        lower_shadow >= (real_body.clone() * factor_2.clone())
            && lower_shadow >= (upper_shadow.clone() * factor_1.clone())
            && down_at_least(prices, DOWN_AT_LEAST_DAYS)
            && price.volume.clone().unwrap() > pre_price.volume.clone().unwrap()
    }

    fn name(&self) -> String {
        "锤子线".to_string()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DojiStarPattern {}

impl StockPattern for DojiStarPattern {
    fn is_match(
        &self,
        stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        _df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        let pre_price = prices.get(prices.len() - 2).unwrap();
        let factor_1 = if stock.stock_type == "Fund" {
            BigDecimal::from_str("0.003").unwrap()
        } else {
            BigDecimal::from_str("0.03").unwrap()
        };
        let real_body = price.get_real_body();
        let lower_shadow = price.get_lower_shadow();
        let upper_shadow = price.get_upper_shadow();
        real_body <= factor_1
            && lower_shadow > upper_shadow
            && down_at_least(prices, DOWN_AT_LEAST_DAYS)
            && price.volume.clone().unwrap() < pre_price.volume.clone().unwrap()
        // 缩量
    }

    fn name(&self) -> String {
        "十字星".to_string()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BullishEngulfingPattern {}
impl StockPattern for BullishEngulfingPattern {
    fn is_match(
        &self,
        _stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        _df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        if price.is_down() {
            return false;
        }

        let factor_1 = BigDecimal::from_str("2").unwrap();
        let real_body = price.get_real_body();
        let upper_shadow = price.get_upper_shadow();

        let pre_price = prices.get(prices.len() - 2);
        if let Some(pre_price) = pre_price {
            let pre_open = &pre_price.open;
            let pre_close = &pre_price.close;
            if pre_price.is_down() {
                let pre_real_body: BigDecimal = pre_price.get_real_body();
                if price.open < pre_close.clone()
                    && price.close > pre_open.clone()
                    && real_body > pre_real_body
                    && real_body > (upper_shadow.clone() * factor_1.clone())
                    && down_at_least(&prices[0..prices.len() - 1], DOWN_AT_LEAST_DAYS)
                    && price.volume.clone().unwrap() > pre_price.volume.clone().unwrap()
                {
                    return true;
                }
            }
        }
        false
    }

    fn name(&self) -> String {
        "看涨吞没".to_string()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PiercingPattern {}

impl StockPattern for PiercingPattern {
    fn is_match(
        &self,
        _stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        _df: &DataFrame,
    ) -> bool {
        if prices.len() < 2 {
            return false;
        }
        let price = prices.last().unwrap();
        let pre_price = prices.get(prices.len() - 2).unwrap();
        let mid_price = pre_price.get_middle_price();
        let factor = BigDecimal::from_str("2").unwrap();
        let real_body = price.get_real_body();
        let upper_shadow = price.get_upper_shadow();

        price.is_up()
            && pre_price.is_down()
            && price.open < pre_price.close
            && price.close > mid_price
            && price.close < pre_price.open
            && real_body > (upper_shadow.clone() * factor.clone())
            && down_at_least(&prices[0..prices.len() - 1], DOWN_AT_LEAST_DAYS)
            && price.volume.clone().unwrap() > pre_price.volume.clone().unwrap()
    }

    fn name(&self) -> String {
        "刺透形态".to_string()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RisingWindowPattern {}

impl StockPattern for RisingWindowPattern {
    fn is_match(
        &self,
        _stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        _df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        let pre_price = prices.get(prices.len() - 2).unwrap();
        let factor = BigDecimal::from_str("2").unwrap();
        let real_body = price.get_real_body();
        let upper_shadow = price.get_upper_shadow();
        price.is_up()
            // && pre_price.is_down()
            && price.open > pre_price.high
            && real_body > (upper_shadow.clone() * factor.clone())
            && down_at_least(&prices[0..prices.len() - 1], DOWN_AT_LEAST_DAYS)
            && price.volume.clone().unwrap() > pre_price.volume.clone().unwrap()
    }

    fn name(&self) -> String {
        "缺口向上".to_string()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MaPattern {
    pub ma: usize,
}

impl StockPattern for MaPattern {
    fn is_match(
        &self,
        _stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        // let pre_price = prices.get(prices.len() - 2).unwrap();
        let n = self.ma;
        let close_df = df
            .clone()
            .lazy()
            .select([col("close").cast(DataType::Float32)])
            .collect()
            .unwrap();
        let ma = ma(&close_df["close"], n);
        let ma_last = ma.last().unwrap();
        // let ma_last_pre = ma.get(ma.len() - 2).unwrap();
        price.close >= BigDecimal::from_f32(*ma_last).unwrap()
        // && pre_price.close <= BigDecimal::from_f32(*ma_last_pre).unwrap()
    }

    fn name(&self) -> String {
        format!("MA{}", &self.ma)
    }
}

pub fn get_candlestick_patterns() -> Vec<Box<dyn StockPattern>> {
    vec![
        Box::new(HammerPattern {}),
        Box::new(DojiStarPattern {}),
        Box::new(BullishEngulfingPattern {}),
        Box::new(PiercingPattern {}),
        Box::new(RisingWindowPattern {}),
    ]
}

pub fn get_ma_patterns() -> Vec<Box<dyn StockPattern>> {
    vec![
        Box::new(MaPattern { ma: 60 }),
        Box::new(MaPattern { ma: 120 }),
    ]
}
