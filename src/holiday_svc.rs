use std::error::Error;

use chrono::{DateTime, Local};
use context::SERVICES;
use database::DbService;

use crate::holiday::MarketHoliday;

pub async fn is_holiday(date: &DateTime<Local>) -> Result<bool, Box<dyn Error>> {
    let date = date.format("%Y%m%d").to_string();

    let dao = SERVICES.get::<DbService>().dao();

    let market_holiday = MarketHoliday::select_by_id(dao, date.parse::<u64>().unwrap()).await?;
    match market_holiday {
        None => { Ok(true) }
        Some(_) => { Ok(false) }
    }
}
