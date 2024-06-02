use std::error::Error;

use context::SERVICES;
use database::DbService;

use crate::index::stock_index::{IndexConstituent, StockIndex};
use crate::index::stock_index_api;
use crate::stock::stock_svc::sync_stock_daily_price;

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

pub async fn sync_constituent_stocks_daily_price(index: &str) -> Result<(), Box<dyn Error>> {
    let stocks = get_constituent_stocks(index).await?;
    for stock in stocks {
        let _ = sync_stock_daily_price(&stock.stock_code).await;
    }
    Ok(())
}

pub async fn get_all_stock_index() -> Result<Vec<StockIndex>, Box<dyn Error>> {
    let dao = SERVICES.get::<DbService>().dao();
    let indexes = StockIndex::select_all(dao).await?;
    Ok(indexes)
}
