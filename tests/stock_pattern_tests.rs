#[cfg(test)]
mod tests {
    use rbatis::rbdc::Decimal;

    use stock_bot::stock_model::StockDailyPrice;
    use stock_bot::stock_pattern::{get_stock_pattern, StockPattern};

    #[test]
    fn test_get_stock_pattern() {
        let price = StockDailyPrice {
            code: "600916".to_string(),
            high: Decimal::new("11.26").unwrap(),
            low: Decimal::new("10.94").unwrap(),
            open: Decimal::new("11.24").unwrap(),
            amount: Some(Decimal::new("300").unwrap()),
            volume: None,
            zf: None,
            hs: None,
            zd: None,
            date: 0,
            close: Decimal::new("11.26").unwrap(),
            zde: None,
        };
        let pattern = get_stock_pattern(&price);
        assert_eq!(StockPattern::LongLowerShadow, pattern);

        let price = StockDailyPrice {
            code: "600916".to_string(),
            high: Decimal::new("11.26").unwrap(),
            low: Decimal::new("11.24").unwrap(),
            open: Decimal::new("11.25").unwrap(),
            amount: Some(Decimal::new("300").unwrap()),
            volume: None,
            zf: None,
            hs: None,
            zd: None,
            date: 0,
            close: Decimal::new("11.26").unwrap(),
            zde: None,
        };
        let pattern = get_stock_pattern(&price);
        assert_eq!(StockPattern::CrossStar, pattern);
    }
}
