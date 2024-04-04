use std::error::Error;
use context::SERVICES;
use database::DbService;
use rbatis::{crud, impl_select};
use serde::{Deserialize, Serialize};
use crate::stock_api;

#[derive(Debug, Serialize, Deserialize)]
pub struct Stock {
    pub code: String,
    pub name: String,
    pub exchange: String,
}
crud!(Stock {});
impl_select!(Stock {select_by_code(code: &str) -> Option => "`where code = #{code}`"});

const COLUMN_CODE: &str = "code";

pub async fn init_stocks() -> Result<(), Box<dyn Error>> {
    let stocks = stock_api::get_stocks().await?;

    let db = SERVICES.get::<DbService>().dao();
    for stock_dto in stocks {
        let stock = Stock::select_by_code(db, &stock_dto.dm).await?;
        let s = stock_dto.clone();
        if stock.is_none() {
            Stock::insert(db, &Stock {
                code: s.dm,
                name: s.mc,
                exchange: s.jys,
            }).await?;
        } else {
            Stock::update_by_column(db, &Stock {
                code: s.dm,
                name: s.mc,
                exchange: s.jys,
            }, COLUMN_CODE).await?;
        }
    }

    Ok(())
}
