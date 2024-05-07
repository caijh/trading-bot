use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::path::Path;

use calamine::{open_workbook, Reader, Xls, Xlsx};
use context::SERVICES;
use database::DbService;
use rbatis::rbdc::Decimal;
use util::request::Request;

use crate::exchange::Exchange;
use crate::stock::{Stock, StockDailyPrice, StockDailyPriceSyncRecord, StockPrice};
use crate::stock_api;

pub async fn sync_stocks(exchange: &Exchange) -> Result<(), Box<dyn Error>> {
    match exchange {
        Exchange::SH(exchange) => {
            let url = "http://query.sse.com.cn/sseQuery/commonExcelDd.do?sqlId=COMMON_SSE_CP_GPJCTPZ_GPLB_GP_L&type=inParams&CSRC_CODE=&STOCK_CODE=&REG_PROVINCE=&STOCK_TYPE=1&COMPANY_STATUS=2,4,5,7,8";
            download(url, Path::new("sh_stocks.xls")).await?;
            let stocks = read_stocks_from_hz_excel("sh_stocks.xls", exchange)?;
            delete_stocks(exchange).await?;
            save_stocks(stocks).await?;
        }
        Exchange::SZ(exchange) => {
            let url = "https://www.szse.cn/api/report/ShowReport?SHOWTYPE=xlsx&CATALOGID=1110&TABKEY=tab1&random=0.4030052742011667";
            Request::download(url, Path::new("sz_stocks.xlsx")).await?;
            let stocks = read_stocks_from_sz_excel("sz_stocks.xlsx", exchange)?;
            delete_stocks(exchange).await?;
            save_stocks(stocks).await?;
        }
    }
    Ok(())
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

pub fn read_stocks_from_hz_excel(path: &str, exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
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
            });
        }
    }

    Ok(stocks)
}

pub fn read_stocks_from_sz_excel(path: &str, exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
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
async fn save_stocks(stocks: Vec<Stock>) -> Result<(), Box<dyn Error>> {
    let db = SERVICES.get::<DbService>().dao();

    Stock::insert_batch(db, &stocks, stocks.len() as u64).await?;

    Ok(())
}

async fn delete_stocks(exchange: &str) -> Result<(), Box<dyn Error>> {
    let dao = SERVICES.get::<DbService>().dao();

    Stock::delete_by_column(dao, "exchange", exchange).await?;

    Ok(())
}

pub async fn get_stock_daily_price(code: &str) -> Result<Vec<StockDailyPrice>, Box<dyn Error>> {
    let dao = SERVICES.get::<DbService>().dao();
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
    if let Some(stock_daily_price_sync_record) =
        StockDailyPriceSyncRecord::select_by_code_date(dao, code, date).await?
    {
        updated = stock_daily_price_sync_record.updated;
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
        StockDailyPriceSyncRecord::insert(
            dao,
            &StockDailyPriceSyncRecord {
                code: code.to_string(),
                date,
                updated: true,
            },
        )
        .await?;
    }

    Ok(daily_prices)
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
