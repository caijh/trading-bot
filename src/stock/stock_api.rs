use std::error::Error;
use std::str::FromStr;

use chrono::{Local, NaiveDateTime};
use configuration::Configuration;
use context::SERVICES;
use database::DbService;
use rand::{thread_rng, Rng};
use rbatis::rbatis_codegen::ops::AsProxy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use util::request::Request;

use crate::exchange::exchange_model::Exchange;
use crate::stock::stock_model::Stock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockDTO {
    pub dm: String,
    pub mc: String,
    pub jys: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockDailyPriceDTO {
    pub d: String,
    pub o: String,
    pub h: String,
    pub l: String,
    pub c: String,
    pub v: String,
    pub e: String,
    pub zd: String,
    pub zdf: String,
    pub hs: String,
}

pub async fn get_stock_daily_price(
    stock: &Stock,
) -> Result<Vec<StockDailyPriceDTO>, Box<dyn Error>> {
    let exchange = Exchange::from_str(stock.exchange.as_str()).unwrap();
    let config = Configuration::get_config().await;
    match exchange {
        Exchange::SH(_) => {
            let url = config.get_string("stock.api.sh.baseurl").unwrap();
            let url = format!(
                "{}/v1/sh1/dayk/{}?begin=-1000&end=-1&period=day&_={}",
                url,
                &stock.code,
                Local::now().timestamp_millis()
            );
            let response = Request::get_response(&url).await?;
            let json: Value = response.json().await?;
            let kline = json.get("kline").unwrap().as_array();
            let mut stock_prices = Vec::new();
            if let Some(kline) = kline {
                for k in kline {
                    let k = k.as_array().unwrap();
                    let price = StockDailyPriceDTO {
                        d: k.first().unwrap().to_string(),
                        o: k.get(1).unwrap().to_string(),
                        h: k.get(2).unwrap().to_string(),
                        l: k.get(3).unwrap().to_string(),
                        c: k.get(4).unwrap().to_string(),
                        v: k.get(5).unwrap().to_string(),
                        e: k.get(6).unwrap().to_string(),
                        zd: "".to_string(),
                        zdf: "".to_string(),
                        hs: "".to_string(),
                    };
                    stock_prices.push(price);
                }
            }
            Ok(stock_prices)
        }
        Exchange::SZ(_) => {
            let url = config.get_string("stock.api.sz.baseurl").unwrap();
            let url = format!(
                "{}/api/market/ssjjhq/getHistoryData?random={}&cycleType=32&marketId=1&code={}",
                url,
                thread_rng().gen::<f64>(),
                &stock.code
            );
            let response = Request::get_response(&url).await?;
            let json: Value = response.json().await?;
            let kline = json
                .get("data")
                .unwrap()
                .get("picupdata")
                .unwrap()
                .as_array();
            let mut stock_prices = Vec::new();
            if let Some(kline) = kline {
                for k in kline {
                    let k = k.as_array().unwrap();
                    let price = StockDailyPriceDTO {
                        d: k.first()
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string()
                            .replace('-', ""),
                        o: k.get(1).unwrap().as_str().unwrap().to_string(),
                        c: k.get(2).unwrap().as_str().unwrap().to_string(),
                        l: k.get(3).unwrap().as_str().unwrap().to_string(),
                        h: k.get(4).unwrap().as_str().unwrap().to_string(),
                        zd: k.get(5).unwrap().as_str().unwrap().to_string(),
                        zdf: k.get(6).unwrap().as_str().unwrap().to_string(),
                        v: k.get(7).unwrap().to_string(),
                        e: k.get(8).unwrap().to_string(),
                        hs: "".to_string(),
                    };
                    stock_prices.push(price);
                }
            }
            Ok(stock_prices)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockPriceDTO {
    // 最高价
    pub h: String,
    // 最低价
    pub l: String,
    // 开盘价
    pub o: String,
    // 涨跌幅（%）
    pub pc: String,
    // 当前价
    pub p: String,
    // 成交额（元）
    pub cje: String,
    // 涨跌额（元）
    pub ud: String,
    // 成交量（手）
    pub v: String,
    // 昨收
    pub yc: String,
    // 时间
    pub t: String,
}

pub async fn get_current_price(code: &str) -> Result<StockPriceDTO, Box<dyn Error>> {
    let db = SERVICES.get::<DbService>().dao();
    let stock = Stock::select_by_code(db, code).await?;
    let stock = match stock {
        Some(s) => s,
        None => return Err("Stock not found".into()),
    };
    let client = Request::client().await;
    let config = Configuration::get_config().await;
    let exchange = Exchange::from_str(&stock.exchange).unwrap();
    match exchange {
        Exchange::SH(_exchange) => {
            let url = config.get_string("stock.api.sh.baseurl").unwrap();
            let response = client
                .get(format!(
                    "{}/v1/sh1/snap/{}?_={}",
                    url,
                    code,
                    Local::now().timestamp_millis()
                ))
                .send()
                .await?;
            let json: Value = response.json().await?;
            let snap = json.get("snap").unwrap();
            let date = json.get("date").unwrap().to_string();
            let time = json.get("time").unwrap().to_string();
            let time = if time.len() < 6 {
                format!("{}{}", 0, time)
            } else {
                time
            };
            Ok(StockPriceDTO {
                h: snap.get(3).unwrap().to_string().string(),
                l: snap.get(4).unwrap().to_string(),
                o: snap.get(2).unwrap().to_string(),
                pc: snap.get(7).unwrap().to_string(),
                p: snap.get(5).unwrap().to_string(),
                cje: snap.get(10).unwrap().to_string(),
                ud: snap.get(8).unwrap().to_string(),
                v: snap.get(9).unwrap().to_string(),
                yc: snap.get(1).unwrap().to_string(),
                t: NaiveDateTime::parse_from_str(&format!("{}{}", date, time), "%Y%m%d%H%M%S")
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            })
        }
        Exchange::SZ(_exchange) => {
            let url = config.get_string("stock.api.sz.baseurl").unwrap();
            let response = client
                .get(format!(
                    "{}/api/market/ssjjhq/getTimeData?random={}&marketId=1&code={}",
                    url,
                    thread_rng().gen::<f64>(),
                    code
                ))
                .send()
                .await?;
            let json: Value = response.json().await?;
            let data = json.get("data").unwrap();
            Ok(StockPriceDTO {
                h: data["high"].as_str().unwrap().to_string(),
                l: data["low"].as_str().unwrap().to_string(),
                o: data["open"].as_str().unwrap().to_string(),
                pc: data["deltaPercent"].as_str().unwrap().to_string(),
                p: data["now"].as_str().unwrap().to_string(),
                cje: data["amount"].as_number().unwrap().to_string(),
                ud: data["delta"].as_str().unwrap().to_string(),
                v: data["volume"].as_number().unwrap().to_string(),
                yc: "".to_string(),
                t: data["marketTime"].as_str().unwrap().to_string(),
            })
        }
    }
}
