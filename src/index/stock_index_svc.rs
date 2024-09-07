use crate::index::stock_index_api;
use crate::index::stock_index_model::{IndexConstituent, StockIndex, SyncIndexConstituents};
use crate::stock::stock_svc::sync_stock_daily_price;
use application::application::APPLICATION_CONTEXT;
use application::context::application_context::ApplicationContext;
use database::DbService;
use std::error::Error;
use std::ops::Not;

/// 获取指数的成分股
///
/// # Arguments
///
/// * `index`: 指数code
///
/// returns: Result<Vec<IndexConstituent, Global>, Box<dyn Error, Global>>
///
/// # Examples
///
/// ```
///
/// ```
pub async fn get_constituent_stocks(index: &str) -> Result<Vec<IndexConstituent>, Box<dyn Error>> {
    let index = get_stock_index(index).await?;
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get::<DbService>().dao();
    let stocks = IndexConstituent::select_by_column(dao, "index_code", &index.code).await?;
    Ok(stocks)
}

pub async fn sync_constituents(index: &str) -> Result<SyncIndexConstituents, Box<dyn Error>> {
    let index = get_stock_index(index).await?;

    let stocks = stock_index_api::get_stocks(&index.code, &index.exchange).await?;

    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get::<DbService>().dao();
    let old_constituents =
        IndexConstituent::select_by_column(dao, "index_code", &index.code).await?;
    let old_constituent_codes = old_constituents
        .iter()
        .map(|c| c.stock_code.clone())
        .collect::<Vec<String>>();

    let mut constituents_to_add = Vec::new();
    let mut stock_codes = Vec::new();
    for stock in stocks {
        if !old_constituent_codes.contains(&stock.code) {
            let constituent = IndexConstituent {
                index_code: index.code.to_string(),
                stock_code: stock.code.to_string(),
                stock_name: stock.name.to_string(),
            };
            constituents_to_add.push(constituent);
        }
        stock_codes.push(stock.code);
    }

    let mut constituents_to_remove = Vec::new();
    for index_constituent in old_constituents {
        if !stock_codes.contains(&index_constituent.stock_code) {
            constituents_to_remove.push(index_constituent);
        }
    }

    if constituents_to_add.is_empty().not() {
        IndexConstituent::insert_batch(dao, &constituents_to_add, constituents_to_add.len() as u64)
            .await?;
    }
    if constituents_to_remove.is_empty().not() {
        for index_constituent in constituents_to_remove.clone() {
            IndexConstituent::delete_by_index_code_stock_code(
                dao,
                &index_constituent.index_code,
                &index_constituent.stock_code,
            )
            .await?;
        }
    }
    Ok(SyncIndexConstituents {
        added: constituents_to_add,
        removed: constituents_to_remove,
    })
}

pub async fn get_stock_index(index: &str) -> Result<StockIndex, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get::<DbService>().dao();
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
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get::<DbService>().dao();
    let indexes = StockIndex::select_all(dao).await?;
    Ok(indexes)
}
