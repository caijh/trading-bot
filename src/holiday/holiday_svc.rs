use crate::holiday::holiday_api::get_holidays;
use crate::holiday::holiday_model;
use application_beans::factory::bean_factory::BeanFactory;
use application_context::context::application_context::APPLICATION_CONTEXT;
use chrono::{DateTime, Datelike, Local};
use database_mysql_seaorm::Dao;
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct HolidayQueryResult {
    pub is_holiday: bool,
}

pub async fn is_holiday(date: &DateTime<Local>) -> Result<HolidayQueryResult, Box<dyn Error>> {
    if date.weekday().number_from_monday() == 6 || date.weekday().number_from_monday() == 7 {
        return Ok(HolidayQueryResult { is_holiday: true });
    }

    let date = date.format("%Y%m%d").to_string();
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();
    let market_holiday = holiday_model::Entity::find_by_id(date.parse::<u64>().unwrap())
        .one(&dao.connection)
        .await?;
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
    for date in dates {
        let id = date.parse::<u64>().unwrap();
        let d = holiday_model::ActiveModel {
            id: Set(id),
            year: Set(date[0..4].parse::<u16>().unwrap()),
            month: Set(date[4..6].parse::<u8>().unwrap()),
            day: Set(date[6..8].parse::<u8>().unwrap()),
        };
        holidays.push(d);
    }
    if holidays.is_empty() {
        return Ok(());
    }
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();
    holiday_model::Entity::delete_many()
        .exec(&dao.connection)
        .await?;
    holiday_model::Entity::insert_many(holidays)
        .exec(&dao.connection)
        .await?;
    Ok(())
}
