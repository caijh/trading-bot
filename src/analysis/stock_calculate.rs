use std::str::FromStr;

use polars::series::Series;
use rbatis::rbdc::Decimal;

use crate::stock::stock_model::StockDailyPrice;

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

pub fn down_at_least(prices: &[StockDailyPrice], n: i32) -> bool {
    let len = prices.len();
    let mut cur = len - 1;
    let mut count = 0;
    loop {
        let p1 = prices.get(cur).unwrap();
        let p2 = prices.get(cur - 1).unwrap();
        if p1.close < p2.close {
            count += 1;
        } else {
            break;
        }
        if count > n {
            break;
        }
        cur -= 1;
    }
    count > n
}

pub fn max(prices: &[StockDailyPrice], n: usize) -> Decimal {
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
pub fn first_max_min(prices: &[StockDailyPrice]) -> (Decimal, Decimal) {
    let mut max = prices.last().unwrap().close.clone();
    let mut min = prices.last().unwrap().close.clone();
    let len = prices.len();
    for i in 0..len {
        let j = len - 1 - i;
        if prices[j].close > max {
            max = prices[j].close.clone();
            if j > 0 && prices[j - 1].close < max {
                break;
            }
        }
    }
    for i in 0..len {
        let j = len - 1 - i;
        if prices[j].close < min {
            min = prices[j].close.clone();
            if j > 0 && prices[j - 1].close > min {
                break;
            }
        }
    }

    (max, min)
}

pub fn min(prices: &[StockDailyPrice], n: usize) -> Decimal {
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

pub fn mean(prices: &[StockDailyPrice], n: usize) -> Decimal {
    let mut total = Decimal::from_str("0").unwrap();
    let len = prices.len();
    let n = if len < n { len } else { n };
    for i in 0..len {
        if i < n {
            total += prices[len - 1 - i].close.clone();
        } else {
            break;
        }
    }
    total / Decimal::from_str(n.to_string().as_str()).unwrap()
}
