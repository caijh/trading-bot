use std::error::Error;

use context::SERVICES;
use database::DbService;

use crate::stock_index::{IndexConstituent, StockIndex};
use crate::stock_index_api;

pub async fn get_constituent_stocks(index: &str) -> Result<Vec<IndexConstituent>, Box<dyn Error>> {
    let index = get_stock_index(index).await?;
    let dao = SERVICES.get::<DbService>().dao();
    let stocks = IndexConstituent::select_by_column(dao, "index_code", &index.code).await?;
    Ok(stocks)
}

pub async fn sync_constituents(index: &str) -> Result<(), Box<dyn Error>> {
    let index = get_stock_index(index).await?;

    let dao = SERVICES.get::<DbService>().dao();
    let stocks = stock_index_api::get_stocks(&index.code, &index.exchange).await?;
    IndexConstituent::delete_by_column(dao, "index_code", &index.code).await?;
    let mut constituents = Vec::new();
    for stock in stocks {
        let constituent = IndexConstituent {
            index_code: index.code.to_string(),
            stock_code: stock.code.to_string(),
            stock_name: stock.name.to_string(),
        };
        constituents.push(constituent);
    }
    IndexConstituent::insert_batch(dao, &constituents, constituents.len() as u64).await?;
    Ok(())
}

pub async fn get_stock_index(index: &str) -> Result<StockIndex, Box<dyn Error>> {
    let dao = SERVICES.get::<DbService>().dao();
    let index = StockIndex::select_by_code(dao, index).await?;
    match index {
        None => Err("Stock index is no Supported".into()),
        Some(index) => Ok(index),
    }
}
