use crate::exchange::exchange_model::Exchange;
use crate::fund::fund_model;
use crate::holiday::holiday_svc::today_is_holiday;
use crate::index::index_api::IndexApi;
use crate::stock::stock_api::StockDailyPriceDTO;
use crate::stock::stock_model::{Model as Stock, Model, StockPrice};
use crate::stock::stock_price_model::Model as StockDailyPrice;
use crate::stock::{stock_api, stock_model, stock_price_model, sync_record_model};
use application_beans::factory::bean_factory::BeanFactory;
use application_cache::CacheManager;
use application_context::context::application_context::APPLICATION_CONTEXT;
use bigdecimal::BigDecimal;
use calamine::{open_workbook, Reader, Xls, Xlsx};
use chrono::{Local, Timelike, Utc};
use database_mysql_seaorm::Dao;
use rand::{thread_rng, Rng};
use redis::Commands;
use redis_io::Redis;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
};
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::str::FromStr;
use tempfile::tempdir;
use tracing::info;
use util::request::Request;

pub async fn sync(exchange: &str) -> Result<(), Box<dyn Error>> {
    let exchange = Exchange::from_str(exchange)?;
    sync_stocks(&exchange).await?;
    sync_funds(&exchange).await?;
    Ok(())
}

pub async fn sync_stocks(exchange: &Exchange) -> Result<(), Box<dyn Error>> {
    let dir = tempdir()?;
    match exchange {
        Exchange::SH => {
            let url = "http://query.sse.com.cn/sseQuery/commonExcelDd.do?sqlId=COMMON_SSE_CP_GPJCTPZ_GPLB_GP_L&type=inParams&CSRC_CODE=&STOCK_CODE=&REG_PROVINCE=&STOCK_TYPE=1,8&COMPANY_STATUS=2,4,5,7,8";
            let path = dir.path().join("sh_stocks.xls");
            download(url, path.as_path()).await?;
            let stocks = read_stocks_from_sh_excel(path.as_path(), exchange.as_ref())?;
            delete_stocks(exchange.as_ref()).await?;
            save_stocks(&stocks).await?;
        }
        Exchange::SZ => {
            let url = format!("http://www.szse.cn/api/report/ShowReport?SHOWTYPE=xlsx&CATALOGID=1110&TABKEY=tab1&random={}", thread_rng().gen::<f64>());
            let path = dir.path().join("sz_stocks.xlsx");
            Request::download(&url, path.as_path()).await?;
            let stocks = read_stocks_from_sz_excel(path.as_path(), exchange.as_ref())?;
            delete_stocks(exchange.as_ref()).await?;
            save_stocks(&stocks).await?;
        }
        Exchange::HK => {
            let stocks = get_stock_from_hk().await?;
            delete_stocks(exchange.as_ref()).await?;
            save_stocks(&stocks).await?;
        }
        Exchange::NASDAQ => {
            let stocks = exchange.get_index_stocks("nasdaq100").await?;
            delete_stocks(exchange.as_ref()).await?;
            save_stocks(&stocks).await?;
        }
    }
    Ok(())
}

async fn get_stock_from_hk() -> Result<Vec<Model>, Box<dyn Error>> {
    let url = format!(
        "https://www.hsi.com.hk/data/schi/rt/index-series/hsi/constituents.do?{}",
        thread_rng().gen_range(1000..9999)
    );
    let response = Request::get_response(&url).await?;
    let data: Value = response.json().await?;
    let index_series_list = data.get("indexSeriesList").unwrap().as_array().unwrap();
    let index_series = index_series_list.first().unwrap().as_object().unwrap();
    let index_list = index_series.get("indexList").unwrap().as_array().unwrap();
    let index_stocks = index_list
        .first()
        .unwrap()
        .get("constituentContent")
        .unwrap()
        .as_array()
        .unwrap();
    let mut stocks = Vec::new();
    for index_stock in index_stocks {
        let stock = Stock {
            code: index_stock
                .get("code")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            name: index_stock
                .get("constituentName")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            exchange: "HK".to_string(),
            stock_type: "Stock".to_string(),
            to_code: None,
        };
        stocks.push(stock);
    }
    Ok(stocks)
}

pub async fn sync_funds(exchange: &Exchange) -> Result<(), Box<dyn Error>> {
    match exchange {
        Exchange::SH => {
            let url = format!(
                "https://query.sse.com.cn/commonSoaQuery.do?sqlId=FUND_LIST&fundType=00&_={}",
                thread_rng().gen::<f64>()
            );
            let stocks = download_funds(&url, exchange.as_ref()).await?;
            delete_funds(exchange.as_ref()).await?;
            save_stocks(&stocks).await?;
            save_funds(&stocks).await?;
        }
        Exchange::SZ => {
            let url = format!("http://www.szse.cn/api/report/ShowReport?SHOWTYPE=xlsx&CATALOGID=1105&TABKEY=tab1&random={}", thread_rng().gen::<f64>());
            let dir = tempdir()?;
            let path_buf = dir.path().join("sz_funds.xlsx");
            Request::download(&url, path_buf.as_path()).await?;
            let stocks = read_funds_from_sz_excel(path_buf.as_path(), exchange.as_ref())?;
            delete_funds(exchange.as_ref()).await?;
            save_stocks(&stocks).await?;
            save_funds(&stocks).await?;
        }
        Exchange::HK => {}
        Exchange::NASDAQ => {}
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
                        stock_type: "Fund".to_string(),
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
                stock_type: "Fund".to_string(),
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
    let dao = application_context.get_bean_factory().get::<Dao>();

    let stocks: Vec<stock_model::ActiveModel> = stocks
        .iter()
        .map(|e| e.clone().into_active_model())
        .collect();
    stock_model::Entity::insert_many(stocks)
        .on_empty_do_nothing()
        .exec(&dao.connection)
        .await?;
    Ok(())
}

async fn save_funds(stocks: &Vec<Stock>) -> Result<(), Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();

    let mut funds = Vec::new();
    for stock in stocks {
        funds.push(fund_model::ActiveModel {
            code: Set(stock.code.clone()),
            name: Set(stock.name.clone()),
            exchange: Set(stock.exchange.clone()),
        });
    }

    fund_model::Entity::insert_many(funds)
        .on_empty_do_nothing()
        .exec(&dao.connection)
        .await?;

    Ok(())
}

pub async fn delete_stocks(exchange: &str) -> Result<(), Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();
    stock_model::Entity::delete_many()
        .filter(stock_model::Column::Exchange.eq(exchange))
        .filter(stock_model::Column::StockType.eq("Stock"))
        .exec(&dao.connection)
        .await?;

    Ok(())
}

pub async fn delete_funds(exchange: &str) -> Result<(), Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();
    fund_model::Entity::delete_many()
        .filter(fund_model::Column::Exchange.eq(exchange))
        .exec(&dao.connection)
        .await?;

    stock_model::Entity::delete_many()
        .filter(stock_model::Column::Exchange.eq(exchange))
        .filter(stock_model::Column::StockType.eq("Fund"))
        .exec(&dao.connection)
        .await?;

    Ok(())
}

pub async fn get_stock_daily_price_from_cache(
    dao: &Dao,
    stock: &Stock,
    date: u64,
) -> Result<Vec<stock_price_model::Model>, Box<dyn Error>> {
    let client = Redis::get_client();
    let mut con = client.get_connection()?;
    let key = "Stock:".to_string() + &stock.code;
    let value = con.get::<&str, Option<String>>(&key)?;
    match value {
        None => {
            let sync_record = sync_record_model::Entity::find_by_id(&stock.code)
                .one(&dao.connection)
                .await?;
            let mut updated: bool = false;
            if let Some(sync_record) = sync_record {
                updated = sync_record.date == date && sync_record.updated;
            }
            let prices = if updated {
                // 从数据库获取
                let prices = stock_price_model::Entity::find()
                    .filter(stock_price_model::Column::Code.eq(&stock.code))
                    .order_by_asc(stock_price_model::Column::Date)
                    .all(&dao.connection)
                    .await?;
                let client = Redis::get_client();
                let mut con = client.get_connection()?;
                let key = "Stock:".to_string() + &stock.code;
                let seconds = 3600 * 24 - Local::now().num_seconds_from_midnight();
                con.set_ex::<&str, String, String>(
                    &key,
                    serde_json::to_string(&prices).unwrap(),
                    seconds as u64,
                )?;
                prices
            } else {
                Vec::new()
            };
            Ok(prices)
        }
        Some(value) => {
            info!("Get stock daily price from cache, code = {}", stock.code);
            let prices: Vec<stock_price_model::Model> = serde_json::from_str(&value).unwrap();
            Ok(prices)
        }
    }
}

pub async fn get_stock_daily_price(code: &str) -> Result<Vec<StockDailyPrice>, Box<dyn Error>> {
    info!("Get stock daily price, code = {}", code);
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();
    let stock = stock_model::Entity::find_by_id(code)
        .one(&dao.connection)
        .await?;
    if stock.is_none() {
        return Err("Stock not found".into());
    }
    let stock = stock.unwrap();
    let date = Local::now()
        .format("%Y%m%d")
        .to_string()
        .parse::<u64>()
        .unwrap();
    let mut daily_prices: Vec<StockDailyPrice> =
        get_stock_daily_price_from_cache(dao, &stock, date).await?;

    if daily_prices.is_empty() {
        let prices_dto = stock_api::get_stock_daily_price(&stock).await?;
        for dto in prices_dto {
            let daily_price = create_stock_daily_price(code, &dto);
            daily_prices.push(daily_price);
        }
    }

    Ok(daily_prices)
}

pub async fn sync_stock_daily_price(code: &str) -> Result<(), Box<dyn Error>> {
    let stock = get_stock(code).await?;
    let exchange = Exchange::from_str(&stock.exchange)?;
    let date = Utc::now()
        .with_timezone(&exchange.time_zone())
        .format("%Y%m%d")
        .to_string()
        .parse::<u64>()
        .unwrap();
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();
    let sync_record = sync_record_model::Entity::find_by_id(&stock.code)
        .one(&dao.connection)
        .await?;
    // 判断是否已经同步
    let mut updated = false;
    if let Some(sync_record) = sync_record {
        updated = sync_record.date == date && sync_record.updated;
    } else {
        let record = sync_record_model::ActiveModel {
            code: Set(code.to_string()),
            date: Set(date),
            updated: Set(false),
        };
        sync_record_model::Entity::insert(record)
            .on_empty_do_nothing()
            .exec(&dao.connection)
            .await?;
    }
    info!("Sync stock {} daily price, updated = {}", code, updated);
    if !updated {
        // 从数据中获取
        let prices = stock_price_model::Entity::find()
            .filter(stock_price_model::Column::Code.eq(&stock.code))
            .order_by_asc(stock_price_model::Column::Date)
            .all(&dao.connection)
            .await?;
        let last_price = if !prices.is_empty() {
            prices.last()
        } else {
            None
        };
        let dates: Vec<u64> = prices.iter().map(|e| e.date).collect();
        let mut new_prices = Vec::new();
        let mut price_dates = Vec::new();
        let prices_dto = stock_api::get_stock_daily_price(&stock).await?;
        for dto in prices_dto {
            let daily_price = create_stock_daily_price(code, &dto);
            let d = daily_price.date;
            price_dates.push(d);

            if !dates.contains(&d) {
                // 数据库中没有
                new_prices.push(daily_price.clone().into_active_model());
            }

            if stock.exchange == "HK" && last_price.is_some() && last_price.unwrap().date == d {
                // 港交所今天的数据，要到明天才更新
                let price = daily_price.clone().into_active_model();
                price.update(&dao.connection).await?;
            }
        }
        if !new_prices.is_empty() {
            stock_price_model::Entity::insert_many(new_prices)
                .exec(&dao.connection)
                .await?;
        }
        if price_dates.contains(&date) || today_is_holiday().await?.is_holiday {
            let record = sync_record_model::ActiveModel {
                code: Set(code.to_string()),
                date: Set(date),
                updated: Set(true),
            };
            sync_record_model::Entity::update(record)
                .filter(sync_record_model::Column::Code.eq(code.to_string()))
                .exec(&dao.connection)
                .await?;
        }
    }
    Ok(())
}

fn create_stock_daily_price(code: &str, dto: &StockDailyPriceDTO) -> StockDailyPrice {
    StockDailyPrice {
        code: code.to_string(),
        date: dto.d.parse::<u64>().unwrap(),
        open: BigDecimal::from_str(&dto.o).unwrap(),
        close: BigDecimal::from_str(&dto.c).unwrap(),
        high: BigDecimal::from_str(&dto.h).unwrap(),
        low: BigDecimal::from_str(&dto.l).unwrap(),
        volume: Some(BigDecimal::from_str(&dto.v).unwrap()),
        amount: if dto.e.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&dto.e).unwrap())
        },
        zf: None,
        hs: None,
        zd: None,
        zde: None,
    }
}

pub async fn get_stock_price(code: &str) -> Result<StockPrice, Box<dyn Error>> {
    let price_dto = stock_api::get_current_price(code).await?;

    let price = StockPrice {
        code: code.to_string(),
        high: if price_dto.h.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&price_dto.h).unwrap())
        },
        low: if price_dto.l.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&price_dto.l).unwrap())
        },
        open: if price_dto.o.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&price_dto.o).unwrap())
        },
        pc: if price_dto.pc.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&price_dto.pc).unwrap())
        },
        price: BigDecimal::from_str(&price_dto.p).unwrap(),
        amount: if price_dto.cje.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&price_dto.cje).unwrap())
        },
        ud: if price_dto.ud.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&price_dto.ud).unwrap())
        },
        volume: if price_dto.v.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&price_dto.v).unwrap())
        },
        yc: if price_dto.yc.is_empty() {
            None
        } else {
            Some(BigDecimal::from_str(&price_dto.yc).unwrap())
        },
        zf: None,
        zs: None,
        time: price_dto.t.clone(),
    };

    Ok(price)
}

pub async fn get_stock(code: &str) -> Result<Stock, Box<dyn Error>> {
    let stock = CacheManager::get(code).await;
    if stock.is_none() {
        let application_context = APPLICATION_CONTEXT.read().await;
        let dao = application_context.get_bean_factory().get::<Dao>();
        let stock = stock_model::Entity::find_by_id(code)
            .one(&dao.connection)
            .await?;
        match stock {
            Some(stock) => {
                CacheManager::set(code, &serde_json::to_string(&stock).unwrap()).await;
                Ok(stock)
            }
            None => Err(format!("Stock {} not found or not support", code).into()),
        }
    } else {
        let stock = serde_json::from_str(&stock.unwrap()).unwrap();
        Ok(stock)
    }
}
