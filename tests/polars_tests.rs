#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use polars::io::SerReader;
    use polars::prelude::{col, IntoLazy, JsonReader};
    use trading_bot::analysis::stock_calculate::ma;

    #[test]
    fn test_polars() {
        let data = r#"[
        {"date": "1996-12-16T00:00:00.000", "open": 16.86, "close": 16.86, "high": 16.86, "low": 16.86, "volume": 62442.0, "turnover": 105277000.0},
        {"date": "1996-12-17T00:00:00.000", "open": 15.17, "close": 15.17, "high": 16.79, "low": 15.17, "volume": 463675.0, "turnover": 718902016.0},
        {"date": "1996-12-18T00:00:00.000", "open": 15.28, "close": 16.69, "high": 16.69, "low": 15.18, "volume": 445380.0, "turnover": 719400000.0},
        {"date": "1996-12-19T00:00:00.000", "open": 17.01, "close": 16.4, "high": 17.9, "low": 15.99, "volume": 572946.0, "turnover": 970124992.0}
    ]"#;

        let res = JsonReader::new(Cursor::new(data)).finish();
        println!("{:?}", res);
        assert!(res.is_ok());
        let df = res.unwrap();

        let df = df.clone().lazy().select([col("close")]).collect().unwrap();
        let vec = ma(&df["close"], 2);
        println!("{:?}", vec);
    }
}
