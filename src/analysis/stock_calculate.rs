use crate::stock::stock_price_model::{KLine, Model as StockDailyPrice};
use bigdecimal::BigDecimal;
// use polars::datatypes::DataType;
use polars::frame::DataFrame;
// use polars::prelude::{col, IntoLazy};
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
pub fn first_max_min(_df: &DataFrame, prices: &Vec<StockDailyPrice>) -> (BigDecimal, BigDecimal) {
    // let close_df = df
    //     .clone()
    //     .lazy()
    //     .select([col("close").cast(DataType::Float32)])
    //     .collect()
    //     .unwrap();
    // let ma5 = ma(&close_df["close"], 5);

    let max = find_first_resistance(prices);
    let max_price = prices.get(max).unwrap().high.clone();


    let min = find_first_support(prices);
    let min_price = prices.get(min).unwrap().low.clone();
    (max_price, min_price)
}

pub fn find_first_resistance(prices: &Vec<StockDailyPrice>) -> usize {
    let latest_price = prices.last().unwrap();
    let len = prices.len();
    let last_idx =  len - 1;
    let mut j = last_idx;
    for i in 0..len {
        j = last_idx - i;
        let price = &prices[j];
        if &price.low > &latest_price.close {
            let pre_idx = j - 1;
            let next_idx = j + 1;
            if j > 0 && next_idx <= last_idx {
                let pre_price = &prices[pre_idx];
                let next_price = &prices[next_idx];
                if &pre_price.low > &price.low  && &next_price.low > &price.low {
                    break;
                }
            }
        }
    }
    j
}

pub fn find_first_support(prices: &Vec<StockDailyPrice>) -> usize {
    let latest_price = prices.last().unwrap();
    let len = prices.len();
    let last_idx =  len - 1;
    let mut j = last_idx;
    for i in 0..len {
        j = last_idx - i;
        let price = &prices[j];
        if &price.high < &latest_price.close {
            let pre_idx = j - 1;
            let next_idx = j + 1;
            if j > 0 && next_idx <= last_idx {
                let pre_price = &prices[pre_idx];
                let next_price = &prices[next_idx];
                if &pre_price.high < &price.high  && &next_price.high < &price.high {
                    break;
                }
            }
        }
    }
    j
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
