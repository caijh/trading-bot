use rbatis::crud;
use rbdc_mysql::types::year::Year;
use serde::{Deserialize, Serialize};

/// 休市日期
#[derive(Serialize, Deserialize)]
pub struct MarketHoliday {
    id: u64,
    year: Year,
    month: u8,
    day: u8,
}
crud!(MarketHoliday {});