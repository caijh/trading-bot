use polars::series::Series;

pub fn ma(prices: &Series, n: usize) -> Vec<f32> {
    (0..prices.len())
        .map(|x| -> f32 {
            let m2 = if x < n {
                prices.slice(0, x + 1).mean().unwrap() as f32
            } else {
                prices.slice((x - n) as i64, n).mean().unwrap() as f32
            };
            m2
        })
        .collect()
}
