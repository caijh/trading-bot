use polars::series::Series;

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
        if p1.close <= p2.close {
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
