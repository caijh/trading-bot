use std::error::Error;
use configuration::Configuration;

use context::SERVICES;
use database::DbService;
use rbatis::{crud, impl_select};
use serde::{Deserialize, Serialize};
use util::request::Request;

#[derive(Debug, Serialize, Deserialize)]
pub struct Stock {
    pub code: String,
    pub name: String,
    pub exchange: String,
}
crud!(Stock {});
impl_select!(Stock {select_by_code(code: &str) -> Option => "`where code = #{code}`"});


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockDTO {
    pub dm: String,
    pub mc: String,
    pub jys: String,
}

const COLUMN_CODE: &str = "code";

pub async fn init_stocks() -> Result<(), Box<dyn Error>> {
    let client = Request::client().await;
    let config = Configuration::get_config().await;
    let url = config.get_string("stock.base_url").unwrap();
    let licence = config.get_string("stock.licence").unwrap();
    let response = client.get(format!("{}/hslt/list/{}", url, licence)).send().await?;
    let stocks: Vec<StockDTO> = response.json().await.unwrap();

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
