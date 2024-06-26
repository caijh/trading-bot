use std::str::FromStr;

use polars::series::Series;
use rbatis::rbdc::Decimal;

use crate::stock::stock_model::StockDailyPrice;

pub fn ma(prices: &Series, n: usize) -> Vec<f32> {
    (0..prices.len())
        .map(|x| -> f32 {
            if x < n {
                prices.slice(0, x + 1).mean().unwrap() as f32
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
