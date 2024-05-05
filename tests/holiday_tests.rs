#[cfg(test)]
mod tests {
    use stock_bot::holiday_api::get_holidays;

    #[tokio::test]
    async fn test_get_holidays() {
        let dates = get_holidays().await.unwrap();
        assert!(!dates.is_empty());
    }
}
