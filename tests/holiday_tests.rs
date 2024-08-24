#[cfg(test)]
mod tests {
    use trading_bot::holiday::holiday_api::get_holidays;

    #[tokio::test]
    async fn test_get_holidays() {
        let dates = get_holidays().await.unwrap();
        assert!(!dates.is_empty());
    }
}
