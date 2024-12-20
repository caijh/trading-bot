#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDateTime};

    #[test]
    fn test_datetime_parse() {
        let date = "20240225";
        let time = "150000";
        let datetime = date.to_string() + time;
        let datetime = NaiveDateTime::parse_from_str(datetime.as_str(), "%Y%m%d%H%M%S");
        match datetime {
            Ok(dt) => assert_eq!(
                "2024-02-25 15:00:00",
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            ),
            Err(err) => panic!("Problem parsing the datetime: {:?}", err),
        };
    }

    #[test]
    fn test_format_timestamp() {
        let dt = DateTime::from_timestamp_millis(1734537600000).unwrap();
        let d = dt
            .with_timezone(&chrono::Local)
            .format("%Y%m%d")
            .to_string();
        assert_eq!(d, "20241219");
    }
}
