use std::error::Error;

use chrono::{DateTime, Datelike, Local};
use context::SERVICES;
use database::DbService;
use serde::{Deserialize, Serialize};

use crate::holiday::holiday_api::get_holidays;
use crate::holiday::holiday_model::MarketHoliday;

#[derive(Serialize, Deserialize, Clone)]
pub struct HolidayQueryResult {
    pub is_holiday: bool,
}

pub async fn is_holiday(date: &DateTime<Local>) -> Result<HolidayQueryResult, Box<dyn Error>> {
    if date.weekday().number_from_monday() == 6 || date.weekday().number_from_monday() == 7 {
        return Ok(HolidayQueryResult { is_holiday: true });
    }

    let date = date.format("%Y%m%d").to_string();
    let dao = SERVICES.get::<DbService>().dao();
    let market_holiday = MarketHoliday::select_by_id(dao, date.parse::<u64>().unwrap()).await?;
    match market_holiday {
        Some(_) => Ok(HolidayQueryResult { is_holiday: true }),
        None => Ok(HolidayQueryResult { is_holiday: false }),
    }
}

pub async fn today_is_holiday() -> Result<HolidayQueryResult, Box<dyn Error>> {
    let now = Local::now();
    is_holiday(&now).await
}

pub async fn sync_holidays() -> Result<(), Box<dyn Error>> {
    let dates = get_holidays().await?;
    let mut holidays = Vec::new();
    let mut ids = Vec::new();
    for date in dates {
        let id = date.parse::<u64>().unwrap();
        ids.push(id);
        let d = MarketHoliday {
            id,
            year: date[0..4].parse::<u16>().unwrap(),
            month: date[4..6].parse::<u8>().unwrap(),
            day: date[6..8].parse::<u8>().unwrap(),
        };
        holidays.push(d);
    }
    if ids.is_empty() {
        return Ok(());
    }
    let dao = SERVICES.get::<DbService>().dao();
    MarketHoliday::delete_in_column(dao, "id", &ids).await?;
    MarketHoliday::insert_batch(dao, &holidays, holidays.len() as u64).await?;
    Ok(())
}
