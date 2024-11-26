use application_beans::factory::bean_factory::BeanFactory;
use application_cache::CacheManager;
use application_context::context::application_context::APPLICATION_CONTEXT;
use calamine::{open_workbook, Reader, Xls, Xlsx};
use database::DbService;
use rand::{thread_rng, Rng};
use rbatis::rbdc::Decimal;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::str::FromStr;
use tempfile::tempdir;
use tracing::info;
use util::request::Request;

use crate::exchange::exchange_model::Exchange;
use crate::fund::fund_model::Fund;
use crate::stock::stock_api;
use crate::stock::stock_model::{Stock, StockDailyPrice, StockDailyPriceSyncRecord, StockPrice};

pub async fn sync(exchange: &str) -> Result<(), Box<dyn Error>> {
    let exchange = Exchange::from_str(exchange)?;
    sync_stocks(&exchange).await?;
    sync_funds(&exchange).await?;
    Ok(())
}

pub async fn sync_stocks(exchange: &Exchange) -> Result<(), Box<dyn Error>> {
    let dir = tempdir()?;
    match exchange {
        Exchange::SH(exchange) => {
            let url = "http://query.sse.com.cn/sseQuery/commonExcelDd.do?sqlId=COMMON_SSE_CP_GPJCTPZ_GPLB_GP_L&type=inParams&CSRC_CODE=&STOCK_CODE=&REG_PROVINCE=&STOCK_TYPE=1,8&COMPANY_STATUS=2,4,5,7,8";
            let path = dir.path().join("sh_stocks.xls");
            download(url, path.as_path()).await?;
            let stocks = read_stocks_from_sh_excel(path.as_path(), exchange)?;
            delete_stocks(exchange).await?;
            save_stocks(&stocks).await?;
        }
        Exchange::SZ(exchange) => {
            let url = format!("https://www.szse.cn/api/report/ShowReport?SHOWTYPE=xlsx&CATALOGID=1110&TABKEY=tab1&random={}", thread_rng().gen::<f64>());
            let path = dir.path().join("sz_stocks.xlsx");
            Request::download(&url, path.as_path()).await?;
            let stocks = read_stocks_from_sz_excel(path.as_path(), exchange)?;
            delete_stocks(exchange).await?;
            save_stocks(&stocks).await?;
        }
    }
    Ok(())
}

pub async fn sync_funds(exchange: &Exchange) -> Result<(), Box<dyn Error>> {
    match exchange {
        Exchange::SH(exchange) => {
            let url = format!(
                "https://query.sse.com.cn/commonSoaQuery.do?sqlId=FUND_LIST&fundType=00&_={}",
                thread_rng().gen::<f64>()
            );
            let stocks = download_funds(&url, exchange).await?;
            delete_funds(exchange.as_ref()).await?;
            save_stocks(&stocks).await?;
            save_funds(&stocks).await?;
        }
        Exchange::SZ(exchange) => {
            let url = format!("https://www.szse.cn/api/report/ShowReport?SHOWTYPE=xlsx&CATALOGID=1105&TABKEY=tab1&random={}", thread_rng().gen::<f64>());
            let dir = tempdir()?;
            let path_buf = dir.path().join("sz_funds.xlsx");
            Request::download(&url, path_buf.as_path()).await?;
            let stocks = read_funds_from_sz_excel(path_buf.as_path(), exchange)?;
            delete_funds(exchange.as_ref()).await?;
            save_stocks(&stocks).await?;
            save_funds(&stocks).await?;
        }
    }
    Ok(())
}

pub async fn download_funds(url: &str, exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
    headers.insert("Referer", "https://www.sse.com.cn/".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    let client = reqwest::Client::builder().build().unwrap();
    let response = client.get(url).headers(headers).send().await;
    match response {
        Ok(response) => {
            let json: Value = response.json().await?;
            let data = json
                .get("pageHelp")
                .unwrap()
                .get("data")
                .unwrap()
                .as_array();
            let mut funds = Vec::new();
            if let Some(data) = data {
                for fund in data {
                    let stock = Stock {
                        code: fund.get("fundCode").unwrap().as_str().unwrap().to_string(),
                        name: fund
                            .get("secNameFull")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string(),
                        exchange: exchange.to_string(),
                        stock_type: "Stock".to_string(),
                        to_code: None,
                    };
                    funds.push(stock);
                }
            }
            Ok(funds)
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn download(url: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
    headers.insert(
        "Referer",
        "http://www.sse.com.cn/assortment/stock/list/share/"
            .parse()
            .unwrap(),
    );
    headers.insert("Connection", "keep-alive".parse().unwrap());
    let client = reqwest::Client::builder().build().unwrap();
    let response = client.get(url).headers(headers).send().await;
    match response {
        Ok(response) => {
            let bytes = response.bytes().await?;
            let mut file = File::create(path)?;
            copy(&mut bytes.as_ref(), &mut file)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn read_stocks_from_sh_excel(
    path: &Path,
    exchange: &str,
) -> Result<Vec<Stock>, Box<dyn Error>> {
    let mut excel_xls: Xls<_> = open_workbook(path)?;

    let mut stocks = Vec::new();
    if let Ok(r) = excel_xls.worksheet_range("股票") {
        for row in r.rows() {
            if row[0] == "A股代码" {
                // 跳过标题行
                continue;
            }
            stocks.push(Stock {
                code: row[0].to_string(),
                name: row[2].to_string(),
                exchange: exchange.to_string(),
                stock_type: "Stock".to_string(),
                to_code: None,
            });
        }
    }

    Ok(stocks)
}

pub fn read_stocks_from_sz_excel(
    path: &Path,
    exchange: &str,
) -> Result<Vec<Stock>, Box<dyn Error>> {
    let mut excel_xlsx: Xlsx<_> = open_workbook(path)?;

    let mut stocks = Vec::new();
    if let Ok(r) = excel_xlsx.worksheet_range("A股列表") {
        for row in r.rows() {
            if row[0] == "板块" {
                // 跳过标题行
                continue;
            }
            stocks.push(Stock {
                code: row[4].to_string(),
                name: row[5].to_string(),
                exchange: exchange.to_string(),
                stock_type: "Stock".to_string(),
                to_code: None,
            });
        }
    }

    Ok(stocks)
}

pub fn read_funds_from_sz_excel(path: &Path, exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
    let mut excel_xlsx: Xlsx<_> = open_workbook(path)?;

    let mut stocks = Vec::new();
    if let Ok(r) = excel_xlsx.worksheet_range("基金列表") {
        for row in r.rows() {
            if row[0] == "基金代码" {
                // 跳过标题行
                continue;
            }
            stocks.push(Stock {
                code: row[0].to_string(),
                name: row[1].to_string(),
                exchange: exchange.to_string(),
                stock_type: "Stock".to_string(),
                to_code: None,
            });
        }
    }

    Ok(stocks)
}

/// 保存或更新股票列表
///
/// # Arguments
///
/// * `stocks`:
///
/// returns: Result<(), Box<dyn Error, Global>>
///
/// # Examples
///
/// ```
///
/// ```
async fn save_stocks(stocks: &[Stock]) -> Result<(), Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context
        .get_bean_factory()
        .get::<DbService>()
        .dao();

    Stock::insert_batch(dao, stocks, stocks.len() as u64).await?;

    Ok(())
}

async fn save_funds(stocks: &Vec<Stock>) -> Result<(), Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context
        .get_bean_factory()
        .get::<DbService>()
        .dao();

    let mut funds = Vec::new();
    for stock in stocks {
        funds.push(Fund {
            code: stock.code.clone(),
            name: stock.name.clone(),
            exchange: stock.exchange.clone(),
        });
    }

    Fund::insert_batch(dao, &funds, funds.len() as u64).await?;

    Ok(())
}

pub async fn delete_stocks(exchange: &str) -> Result<(), Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context
        .get_bean_factory()
        .get::<DbService>()
        .dao();

    Stock::delete_by_exchange(dao, exchange).await?;

    Ok(())
}

pub async fn delete_funds(exchange: &str) -> Result<(), Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context
        .get_bean_factory()
        .get::<DbService>()
        .dao();

    Fund::delete_by_column(dao, "exchange", exchange).await?;

    Ok(())
}

pub async fn get_stock_daily_price(code: &str) -> Result<Vec<StockDailyPrice>, Box<dyn Error>> {
    info!("get_stock_daily_price code = {}", code);
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context
        .get_bean_factory()
        .get::<DbService>()
        .dao();
    let stock = Stock::select_by_code(dao, code).await?;
    if stock.is_none() {
        return Err("Stock not found".into());
    }
    let stock = stock.unwrap();
    let date = chrono::Local::now()
        .format("%Y%m%d")
        .to_string()
        .parse::<u64>()
        .unwrap();
    let mut daily_prices: Vec<StockDailyPrice> =
        StockDailyPrice::select_by_column(dao, "code", code).await?;
    let mut updated: bool = false;
    let sync_record = StockDailyPriceSyncRecord::select_by_code(dao, code).await?;
    if let Some(sync_record) = sync_record {
        updated = sync_record.date == date && sync_record.updated;
    } else {
        StockDailyPriceSyncRecord::insert(
            dao,
            &StockDailyPriceSyncRecord {
                code: code.to_string(),
                date,
                updated: false,
            },
        )
        .await?;
    }
    if !updated {
        let dates: Vec<u64> = daily_prices.iter().map(|e| e.date).collect();
        let mut prices = Vec::new();
        for dto in stock_api::get_stock_daily_price(&stock).await? {
            let daily_price = StockDailyPrice {
                code: code.to_string(),
                date: dto.d.parse::<u64>().unwrap(),
                open: Decimal::new(&dto.o).unwrap(),
                close: Decimal::new(&dto.c).unwrap(),
                high: Decimal::new(&dto.h).unwrap(),
                low: Decimal::new(&dto.l).unwrap(),
                volume: Some(Decimal::new(&dto.v).unwrap()),
                amount: Some(Decimal::new(&dto.e).unwrap()),
                zf: None,
                hs: None,
                zd: None,
                zde: None,
            };
            if !dates.contains(&daily_price.date) {
                prices.push(daily_price.clone());
                daily_prices.push(daily_price);
            }
        }
        if !prices.is_empty() {
            StockDailyPrice::insert_batch(dao, &prices, prices.len() as u64).await?;
        }
        StockDailyPriceSyncRecord::update_by_column(
            dao,
            &StockDailyPriceSyncRecord {
                code: code.to_string(),
                date,
                updated: true,
            },
            "code",
        )
        .await?;
    }

    Ok(daily_prices)
}

pub async fn sync_stock_daily_price(code: &str) -> Result<(), Box<dyn Error>> {
    let _ = get_stock_daily_price(code).await;
    Ok(())
}

pub async fn get_stock_price(code: &str) -> Result<StockPrice, Box<dyn Error>> {
    let price_dto = stock_api::get_current_price(code).await?;

    let price = StockPrice {
        code: code.to_string(),
        high: Some(Decimal::new(&price_dto.h).unwrap()),
        low: Some(Decimal::new(&price_dto.l).unwrap()),
        open: Some(Decimal::new(&price_dto.o).unwrap()),
        pc: Some(Decimal::new(&price_dto.pc).unwrap()),
        price: Decimal::new(&price_dto.p).unwrap(),
        amount: Some(Decimal::new(&price_dto.cje).unwrap()),
        ud: Some(Decimal::new(&price_dto.ud).unwrap()),
        volume: Some(Decimal::new(&price_dto.v).unwrap()),
        yc: None,
        zf: None,
        zs: None,
        time: price_dto.t.clone(),
    };

    Ok(price)
}

pub async fn get_stock(code: &str) -> Result<Option<Stock>, Box<dyn Error>> {
    let stock = CacheManager::get(code).await;
    if stock.is_none() {
        let application_context = APPLICATION_CONTEXT.read().await;
        let dao = application_context
            .get_bean_factory()
            .get::<DbService>()
            .dao();
        let stock = Stock::select_by_code(dao, code).await?;
        match stock {
            None => Ok(None),
            Some(stock) => {
                CacheManager::set(code, &serde_json::to_string(&stock).unwrap()).await;
                Ok(Some(stock))
            }
        }
    } else {
        let stock = serde_json::from_str(&stock.unwrap()).unwrap();
        Ok(Some(stock))
    }
}
