use crate::stock::stock_price_model::{KLine, Model as StockDailyPrice};
use bigdecimal::{BigDecimal, FromPrimitive, RoundingMode};
use polars::datatypes::DataType;
use polars::frame::DataFrame;
use polars::prelude::{col, IntoLazy};
use polars::series::Series;
use std::str::FromStr;

pub fn ma(prices: &Series, n: usize) -> Vec<f32> {
    (0..prices.len())
        .map(|x| -> f32 {
            if x < n {
                prices.slice(0, n).mean().unwrap() as f32
            } else {
                prices.slice((x - (n - 1)) as i64, n).mean().unwrap() as f32
            }
        })
        .collect()
}

/// whether the price is down at least n days
///
/// loop through the array from the end to the beginning
/// if a price is smaller than the previous price, count + 1
/// break the loop if the previous price is larger than the current price
///
/// return the count
pub fn down_at_least(prices: &[StockDailyPrice], n: i32) -> bool {
    let len = prices.len();
    let mut cur = len - 1;
    let mut count = 0;
    let n = n - 1;
    loop {
        let p1 = prices.get(cur).unwrap();
        let p2 = prices.get(cur - 1).unwrap();
        if p1.get_middle_price() <= p2.get_middle_price() {
            count += 1;
        } else {
            break;
        }
        if count >= n {
            break;
        }
        cur -= 1;
    }
    count >= n
}

pub fn max(prices: &[StockDailyPrice], n: usize) -> BigDecimal {
    let mut max = prices.last().unwrap().close.clone();
    let len = prices.len();
    let n = if len < n { len } else { n };
    for i in 0..len {
        if i < n {
            if prices[len - 1 - i].close > max {
                max = prices[len - 1 - i].close.clone();
            }
        } else {
            break;
        }
    }
    max
}

/// find the first max and min in the price array
///
/// loop through the array from the end to the beginning
/// if a price is greater than the current max, update the max
/// if a price is smaller than the current min, update the min
/// break the loop if the previous price is larger than the current max
/// or smaller than the current min
///
/// return the first max and min
pub fn first_max_min(prices: &DataFrame) -> (BigDecimal, BigDecimal) {
    let close_df = prices
        .clone()
        .lazy()
        .select([col("close").cast(DataType::Float32)])
        .collect()
        .unwrap();
    let ma5 = ma(&close_df["close"], 5);
    let max = find_first_max(&ma5);
    let min = find_first_min(&ma5);
    (max, min)
}

pub fn find_first_max(prices: &[f32]) -> BigDecimal {
    let mut max = prices.last().unwrap();
    let len = prices.len();
    for i in 0..len {
        let j = len - 1 - i;
        let price = &prices[j];
        if price > max {
            max = price;
            if j > 0 {
                let pre_price = &prices[j - 1];
                if pre_price < max {
                    break;
                }
            }
        }
    }
    BigDecimal::from_f32(*max)
        .unwrap()
        .with_scale_round(2, RoundingMode::Up)
}

pub fn find_first_min(prices: &[f32]) -> BigDecimal {
    let mut min = prices.last().unwrap();
    let len = prices.len();
    for i in 0..len {
        let j = len - 1 - i;
        let price = &prices[j];
        if price < min {
            min = price;
            if j > 0 {
                let pre_price = &prices[j - 1];
                if pre_price > min {
                    break;
                }
            }
        }
    }
    BigDecimal::from_f32(*min)
        .unwrap()
        .with_scale_round(2, RoundingMode::Up)
}

pub fn min(prices: &[StockDailyPrice], n: usize) -> BigDecimal {
    let mut min = prices.last().unwrap().low.clone();
    let len = prices.len();
    let n = if len < n { len } else { n };
    for i in 0..len {
        if i < n {
            if prices[len - 1 - i].low < min {
                min = prices[len - 1 - i].low.clone();
            }
        } else {
            break;
        }
    }
    min
}

pub fn mean(prices: &[StockDailyPrice], n: usize) -> BigDecimal {
    let mut total = BigDecimal::from_str("0").unwrap();
    let len = prices.len();
    let n = if len < n { len } else { n };
    for i in 0..len {
        if i < n {
            total += prices[len - 1 - i].close.clone();
        } else {
            break;
        }
    }
    total / BigDecimal::from_str(n.to_string().as_str()).unwrap()
}
