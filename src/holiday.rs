use rbatis::{crud, impl_select};
use rbdc_mysql::types::year::Year;
use serde::{Deserialize, Serialize};

/// 休市日期
#[derive(Serialize, Deserialize)]
pub struct MarketHoliday {
    pub id: u64,
    pub year: Year,
    pub month: u8,
    pub day: u8,
}
crud!(MarketHoliday {});
impl_select!(MarketHoliday {select_by_id(id: u64) -> Option => "`where id = #{id}`"});
