use std::error::Error;

use chrono::{Datelike, DateTime, Local};
use context::SERVICES;
use database::DbService;
use rbdc_mysql::types::year::Year;
use util::request::Request;

use crate::holiday::MarketHoliday;

pub async fn is_holiday(date: &DateTime<Local>) -> Result<bool, Box<dyn Error>> {
    if date.weekday().number_from_monday() == 6 || date.weekday().number_from_monday() == 7 {
        return Ok(true);
    }

    let date = date.format("%Y%m%d").to_string();
    let dao = SERVICES.get::<DbService>().dao();
    let market_holiday = MarketHoliday::select_by_id(dao, date.parse::<u64>().unwrap()).await?;
    match market_holiday {
        Some(_) => { Ok(true) },
        None => { Ok(false) }
    }
}

pub async fn sync_holidays() -> Result<(), Box<dyn Error>> {
    let url = "https://raw.githubusercontent.com/rainx/cn_stock_holidays/main/cn_stock_holidays/data.txt";
    let response = Request::get_response(url).await?;
    let content = response.text().await?;
    let dates: Vec<&str> = content.split('\n').filter(|s| !s.is_empty()).collect();
    let mut market_holidays = Vec::new();
    let mut ids = Vec::new();
    for date in dates {
        let id = date.parse::<u64>().unwrap();
        ids.push(id);
        let d = MarketHoliday {
            id,
            year: Year(date[0..4].parse::<u16>().unwrap()),
            month: date[4..6].parse::<u8>().unwrap(),
            day: date[6..8].parse::<u8>().unwrap(),
        };
        market_holidays.push(d);
    }
    if ids.is_empty() {
        return Ok(());
    }
    let dao = SERVICES.get::<DbService>().dao();
    MarketHoliday::delete_in_column(dao, "id", &ids).await?;
    MarketHoliday::insert_batch(dao, &market_holidays, market_holidays.len() as u64).await?;
    Ok(())
}
