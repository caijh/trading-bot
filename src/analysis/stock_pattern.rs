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
        stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        let pre_price = prices.get(prices.len() - 2).unwrap();
        let real_body = price.get_real_body();
        let lower_shadow = price.get_lower_shadow();
        let upper_shadow = price.get_upper_shadow();
        let n = if stock.stock_type != "Fund" {
            DOWN_AT_LEAST_DAYS
        } else {
            DOWN_AT_LEAST_DAYS + 1
        };

        let volumn_pattern = VolumnMaPattern { ma: 20};
        (lower_shadow.clone() / (lower_shadow + real_body + upper_shadow)) > BigDecimal::from_f32(0.618).unwrap()
            && down_at_least(prices, n)
            && price.volume.clone().unwrap() > pre_price.volume.clone().unwrap()
            && volumn_pattern.is_match(stock, prices, df)
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
        df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        let pre_price = prices.get(prices.len() - 2).unwrap();
        let factor = if stock.stock_type == "Fund" {
            BigDecimal::from_str("0.001").unwrap()
        } else {
            BigDecimal::from_str("0.01").unwrap()
        };
        let real_body = price.get_real_body();
        let lower_shadow = price.get_lower_shadow();
        let upper_shadow = price.get_upper_shadow();
        let n = if stock.stock_type != "Fund" {
            DOWN_AT_LEAST_DAYS
        } else {
            DOWN_AT_LEAST_DAYS + 1
        };
        let volumn_pattern = VolumnMaPattern { ma: 20};
        (real_body.clone() / (lower_shadow.clone() + real_body.clone() + upper_shadow.clone())) <= factor
            && lower_shadow >= upper_shadow
            && down_at_least(prices, n)
            && price.volume.clone().unwrap() > pre_price.volume.clone().unwrap()
            && volumn_pattern.is_match(stock, prices, df)
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
        stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        if price.is_down() {
            return false;
        }

        let factor = BigDecimal::from_str("2").unwrap();
        let real_body = price.get_real_body();
        let upper_shadow = price.get_upper_shadow();

        let pre_price = prices.get(prices.len() - 2);
        let n = if stock.stock_type != "Fund" {
            DOWN_AT_LEAST_DAYS
        } else {
            DOWN_AT_LEAST_DAYS + 1
        };
        if let Some(pre_price) = pre_price {
            let pre_open = &pre_price.open;
            let pre_close = &pre_price.close;
            if pre_price.is_down() {
                let pre_real_body: BigDecimal = pre_price.get_real_body();
                let volumn_pattern = VolumnMaPattern { ma: 20};
                if price.open < pre_close.clone()
                    && price.close > pre_open.clone()
                    && real_body > pre_real_body
                    && real_body > (upper_shadow.clone() * factor.clone())
                    && down_at_least(&prices[0..prices.len() - 1], n)
                    && (price.volume.clone().unwrap()
                        > pre_price.volume.clone().unwrap() * BigDecimal::from_f32(1.2).unwrap())
                    && volumn_pattern.is_match(stock, prices, df)
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
        stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        df: &DataFrame,
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
        let n = if stock.stock_type != "Fund" {
            DOWN_AT_LEAST_DAYS
        } else {
            DOWN_AT_LEAST_DAYS + 1
        };
        let volumn_pattern = VolumnMaPattern { ma: 20};
        price.is_up()
            && pre_price.is_down()
            && price.open < pre_price.close
            && price.close > mid_price
            && price.close < pre_price.open
            && real_body > (upper_shadow.clone() * factor.clone())
            && down_at_least(&prices[0..prices.len() - 1], n)
            && (price.volume.clone().unwrap()
                > pre_price.volume.clone().unwrap() * BigDecimal::from_f32(1.2).unwrap())
            && volumn_pattern.is_match(stock, prices, df)
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
        stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        let pre_price = prices.get(prices.len() - 2).unwrap();
        let factor = BigDecimal::from_str("2").unwrap();
        let real_body = price.get_real_body();
        let upper_shadow = price.get_upper_shadow();
        let n = if stock.stock_type != "Fund" {
            DOWN_AT_LEAST_DAYS
        } else {
            DOWN_AT_LEAST_DAYS + 1
        };
        let volumn_pattern = VolumnMaPattern { ma: 20};
        price.is_up()
            // && pre_price.is_down()
            && price.open > pre_price.high
            && real_body > (upper_shadow.clone() * factor.clone())
            && down_at_least(&prices[0..prices.len() - 1], n)
            && price.volume.clone().unwrap() > pre_price.volume.clone().unwrap()
            && volumn_pattern.is_match(stock, prices, df)
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
        let _pre_price = prices.get(prices.len() - 2).unwrap();
        let n = self.ma;
        let close_df = df
            .clone()
            .lazy()
            .select([col("close").cast(DataType::Float32)])
            .collect()
            .unwrap();
        let ma = ma(&close_df["close"], n);
        let ma_last = ma.last().unwrap();
        let ma_last_pre = ma.get(ma.len() - 2).unwrap();
        // 收盘价在均线之上
        price.close >= BigDecimal::from_f32(*ma_last).unwrap()
            // 均线向上
            && ma_last_pre < ma_last
    }

    fn name(&self) -> String {
        format!("MA{}", &self.ma)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BIASPattern {
    pub ma: usize,
    pub bias: f32,
}

impl StockPattern for BIASPattern {
    fn is_match(
        &self,
        _stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        let close_df = df
            .clone()
            .lazy()
            .select([col("close").cast(DataType::Float32)])
            .collect()
            .unwrap();
        let ma = ma(&close_df["close"], self.ma);
        let ma_last = ma.last().unwrap();
        let ma_last = BigDecimal::from_f32(*ma_last).unwrap();
        price.close < ma_last
            && (((ma_last.clone() - price.close.clone()) / ma_last)
                >= BigDecimal::from_f32(self.bias).unwrap())
    }

    fn name(&self) -> String {
        "BIAS".to_string()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VolumnMaPattern {
    pub ma: usize,
}

impl StockPattern for VolumnMaPattern {
    fn is_match(
        &self,
        _stock: &stock_model::Model,
        prices: &[StockDailyPrice],
        df: &DataFrame,
    ) -> bool {
        let price = prices.last().unwrap();
        let volume_df = df
            .clone()
            .lazy()
            .select([col("volume").cast(DataType::Float32)])
            .collect()
            .unwrap();
        let ma = ma(&volume_df["volume"], self.ma);
        let ma_last = ma.get(ma.len() - 2).unwrap();
        let ma_last = BigDecimal::from_f32(*ma_last).unwrap();
        price.volume.clone().unwrap() >= ma_last
    }

    fn name(&self) -> String {
        "VOL".to_string()
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
        Box::new(BIASPattern { ma: 20, bias: 0.15 }),
    ]
}
