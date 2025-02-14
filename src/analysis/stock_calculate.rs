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
        let p1 = prices.get(cur);
        if p1.is_none() {
            break;
        }
        let p2 = prices.get(cur - 1);
        if p2.is_none() {
            break;
        }
        let p1 = p1.unwrap();
        let p2 = p2.unwrap();
        if p1.get_middle_price() < p2.get_middle_price() || p1.close < p2.close {
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
pub fn first_resistance_support_price(df: &DataFrame, prices: &Vec<StockDailyPrice>) -> (BigDecimal, BigDecimal) {
    let close_df = df
        .clone()
        .lazy()
        .select([col("close").cast(DataType::Float32)])
        .collect()
        .unwrap();
    let ma_prices = ma(&close_df["close"], 5);
    let latest_price = prices.last().unwrap();

    let resistance_indexes = find_resistance_indexes(&ma_prices, latest_price);
    let mut min_resistance_price = latest_price.high.clone();
    if !resistance_indexes.is_empty() {
        min_resistance_price = BigDecimal::from_f32(*ma_prices.get(resistance_indexes[0]).unwrap()).unwrap().with_scale_round(3, RoundingMode::Up);
        for i in resistance_indexes {
            let price = BigDecimal::from_f32(*ma_prices.get(i).unwrap()).unwrap().with_scale_round(3, RoundingMode::Up);
            if price > latest_price.close.clone() && price < min_resistance_price {
                min_resistance_price = price;
            }
        }
    }

    let support_indexes = find_support_indexes(&ma_prices, latest_price);
    let mut max_support_price = latest_price.low.clone();
    if !support_indexes.is_empty() {
        max_support_price = BigDecimal::from_f32(*ma_prices.get(support_indexes[0]).unwrap()).unwrap().with_scale_round(3, RoundingMode::Up);
        for i in support_indexes {
            let price = BigDecimal::from_f32(*ma_prices.get(i).unwrap()).unwrap().with_scale_round(3, RoundingMode::Up);
            if price < latest_price.close.clone() && price > max_support_price  {
                max_support_price = price;
            }
        }
    }

    (min_resistance_price, max_support_price)
}

pub fn find_resistance_indexes(prices: &[f32], latest_price: &StockDailyPrice) -> Vec<usize> {
    let latest_price = latest_price.close.clone();
    let len = prices.len();
    let last_idx =  len - 1;
    let mut j: usize;
    let mut idxes = Vec::new();
    for i in 0..len {
        j = last_idx - i;
        let price = &prices[j];
        let price = BigDecimal::from_f32(*price).unwrap();
        if price > latest_price {
            let pre_idx = j - 1;
            let next_idx = j + 1;
            if j > 0 && next_idx <= last_idx {
                let pre_price = &prices[pre_idx];
                let pre_price = BigDecimal::from_f32(*pre_price).unwrap();
                let next_price = &prices[next_idx];
                let next_price = BigDecimal::from_f32(*next_price).unwrap();
                if pre_price > price && next_price > price {
                    idxes.push(j);
                }
            }
        }
    }
    idxes
}

pub fn find_support_indexes(prices: &[f32], latest_price: &StockDailyPrice) -> Vec<usize> {
    let latest_price = latest_price.close.clone();
    let len = prices.len();
    let last_idx =  len - 1;
    let mut j: usize;
    let mut idxes = Vec::new();
    for i in 0..len {
        j = last_idx - i;
        let price = &prices[j];
        let price = BigDecimal::from_f32(*price).unwrap();
        if price < latest_price {
            let pre_idx = j - 1;
            let next_idx = j + 1;
            if j > 0 && next_idx <= last_idx {
                let pre_price = &prices[pre_idx];
                let pre_price = BigDecimal::from_f32(*pre_price).unwrap();
                let next_price = &prices[next_idx];
                let next_price = BigDecimal::from_f32(*next_price).unwrap();
                if pre_price < price  && next_price < price {
                    idxes.push(j);
                }
            }
        }
    }
    idxes
}


pub fn find_first_max(prices: &[f32]) -> usize {
    let mut max = prices.last().unwrap();
    let len = prices.len();
    let last_idx =  len - 1;
    let mut j = last_idx;
    for i in 0..len {
        j = last_idx - i;
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
    j
}

pub fn find_first_min(prices: &[f32]) -> usize {
    let mut min = prices.last().unwrap();
    let len = prices.len();
    let last_idx = len - 1;
    let mut j = last_idx;
    for i in 0..len {
        j = last_idx - i;
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
    j
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
