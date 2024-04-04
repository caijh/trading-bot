use std::error::Error;

use context::SERVICES;
use database::DbService;
use rbatis::{crud, impl_select};
use rbatis::rbdc::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::stock_api;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockDailyPrice {
    pub code: String,
    pub date: i64,
    pub open: Option<Decimal>,
    pub close: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub zf: Option<Decimal>,
    pub hs: Option<Decimal>,
    pub zd: Option<Decimal>,
    pub zde: Option<Decimal>,
}
crud!(StockDailyPrice {});

#[derive(Debug, Serialize, Deserialize)]
pub struct StockDailyPriceSyncRecord {
    pub code: String,
    pub date: i64,
    #[serde(deserialize_with = "bool_or_int")]
    pub updated: bool,
}

fn bool_or_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
{
    struct BoolOrIntVisitor;

    impl<'de> de::Visitor<'de> for BoolOrIntVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean or an integer")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            Ok(value)
        }

        fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            // Map 0 to false, any other value to true
            Ok(value != 0)
        }
    }

    deserializer.deserialize_any(BoolOrIntVisitor)
}

crud!(StockDailyPriceSyncRecord {});
impl_select!(StockDailyPriceSyncRecord {select_by_code_date(code: &str, date: i64) -> Option => "`where code = #{code} and date = #{date}`"});

pub async fn get_stock_daily_price(code: &str) -> Result<Vec<StockDailyPrice>, Box<dyn Error>> {
    let db = SERVICES.get::<DbService>().dao();
    let date = chrono::Local::now().format("%Y%m%d").to_string().parse::<i64>().unwrap();
    let mut daily_prices: Vec<StockDailyPrice> = StockDailyPrice::select_by_column(db, "code", code).await?;
    let mut updated: bool = false;
    if let Some(stock_daily_price_sync_record) = StockDailyPriceSyncRecord::select_by_code_date(db, code, date).await? {
        updated = stock_daily_price_sync_record.updated;
    }
    if !updated {
        let daily_price_dtos = stock_api::get_stock_daily_price(code).await?;
        let dates: Vec<i64> = daily_prices.iter().map(|e| e.date).collect();
        for dto in daily_price_dtos {
            let daily_price = StockDailyPrice {
                code: code.to_string(),
                date: dto.d.replace('-', "").parse::<i64>().unwrap(),
                open: Some(Decimal::new(&dto.o).unwrap()),
                close: Some(Decimal::new(&dto.c).unwrap()),
                high: Some(Decimal::new(&dto.h).unwrap()),
                low: Some(Decimal::new(&dto.l).unwrap()),
                volume: Some(Decimal::new(&dto.v).unwrap()),
                amount: Some(Decimal::new(&dto.e).unwrap()),
                zf: Some(Decimal::new(&dto.zf).unwrap()),
                hs: Some(Decimal::new(&dto.hs).unwrap()),
                zd: Some(Decimal::new(&dto.zd).unwrap()),
                zde: Some(Decimal::new(&dto.zde).unwrap()),
            };
            if !dates.contains(&daily_price.date) {
                StockDailyPrice::insert(db, &daily_price).await?;
                daily_prices.push(daily_price);
            }
        }
        StockDailyPriceSyncRecord::insert(db, &StockDailyPriceSyncRecord {
            code: code.to_string(),
            date,
            updated: true,
        }).await?;
    }

    Ok(daily_prices)
}
