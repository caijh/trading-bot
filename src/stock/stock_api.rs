use application_context::context::application_context::APPLICATION_CONTEXT;
use application_core::env::property_resolver::PropertyResolver;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::str::FromStr;
use tracing::info;
use util::request::Request;

use crate::exchange::exchange_model::Exchange;
use crate::stock::stock_model;
use crate::stock::stock_svc::get_stock;
use crate::token::token_svc;

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
    stock: &stock_model::Model,
) -> Result<Vec<StockDailyPriceDTO>, Box<dyn Error>> {
    let exchange = Exchange::from_str(stock.exchange.as_str())?;
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.get_environment().await;
    let mut stock_prices = Vec::new();
    match exchange {
        Exchange::SH => {
            info!(
                "Get stock daily price from {}, code = {}",
                exchange.as_ref(),
                stock.code
            );
            let url = environment
                .get_property::<String>("stock.api.sh.baseurl")
                .unwrap();
            let url = format!(
                "{}/v1/sh1/dayk/{}?begin=-1000&end=-1&period=day&_={}",
                url,
                &stock.code,
                Local::now().timestamp_millis()
            );
            let response = Request::get_response(&url).await?;
            let json: Value = response.json().await?;
            let kline = json.get("kline").unwrap().as_array();
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
        Exchange::SZ => {
            info!(
                "Get stock daily price from {}, code = {}",
                exchange.as_ref(),
                stock.code
            );
            let url = environment
                .get_property::<String>("stock.api.sz.baseurl")
                .unwrap();
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
        Exchange::HK => {
            info!(
                "Get stock daily price from {}, code = {}",
                exchange.as_ref(),
                stock.code
            );
            let url = environment
                .get_property::<String>("stock.api.hk.baseurl")
                .unwrap();
            let token = token_svc::get_hkex_token().await;
            let timestramp = Local::now().timestamp_millis();
            let code = format!("{:0>4}.HK", stock.code);
            let url = format!(
                "{}/hkexwidget/data/getchartdata2?hchart=1&span=6&int=5&ric={}&token={}&qid={}&callback=jQuery_{}&_={}",
                url,
                code,
                token,
                timestramp,
                timestramp,
                timestramp,
            );
            let response = Request::get_response(&url).await?;
            let text = response.text().await?;
            let json = remove_jquery_wrapping_fn_call(&text);
            let kline = json
                .get("data")
                .unwrap()
                .get("datalist")
                .unwrap()
                .as_array();
            if let Some(kline) = kline {
                for k in kline {
                    let k = k.as_array().unwrap();
                    let o = k.get(1).unwrap();
                    if o.is_null() {
                        continue;
                    }
                    let o = o.as_number().unwrap().to_string();
                    let dt: DateTime<Utc> =
                        DateTime::from_timestamp_millis(k.first().unwrap().as_i64().unwrap())
                            .unwrap();
                    let price = StockDailyPriceDTO {
                        d: dt.with_timezone(&Local).format("%Y%m%d").to_string(),
                        o,
                        c: k.get(4).unwrap().as_number().unwrap().to_string(),
                        l: k.get(3).unwrap().as_number().unwrap().to_string(),
                        h: k.get(2).unwrap().as_number().unwrap().to_string(),
                        zd: "".to_string(),
                        zdf: "".to_string(),
                        v: k.get(5).unwrap().as_number().unwrap().to_string(),
                        e: k.get(6).unwrap().as_number().unwrap().to_string(),
                        hs: "".to_string(),
                    };
                    stock_prices.push(price);
                }
                // append today price
                // let stock_price = get_current_price(&stock.code).await?;
                // let date = Local::now()
                //     .format("%Y%m%d")
                //     .to_string();
                // stock_prices.push(StockDailyPriceDTO {
                //     d: date,
                //     o: stock_price.o,
                //     h: stock_price.h,
                //     l: stock_price.l,
                //     c: stock_price.p,
                //     v: stock_price.v,
                //     e: stock_price.cje,
                //     zd: stock_price.ud,
                //     zdf: stock_price.pc,
                //     hs: "".to_string(),
                // });
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
    let stock = get_stock(code).await?;
    let code = if let Some(code) = stock.to_code {
        code
    } else {
        code.to_string()
    };
    let client = Request::client().await;
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.get_environment().await;
    let exchange = Exchange::from_str(&stock.exchange)?;
    match exchange {
        Exchange::SH => {
            let url = environment
                .get_property::<String>("stock.api.sh.baseurl")
                .unwrap();
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
                h: snap.get(3).unwrap().to_string(),
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
        Exchange::SZ => {
            let url = environment
                .get_property::<String>("stock.api.sz.baseurl")
                .unwrap();
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
        Exchange::HK => {
            let url = environment
                .get_property::<String>("stock.api.hk.baseurl")
                .unwrap();
            let token = token_svc::get_hkex_token().await;
            let timestamp = Local::now().timestamp_millis();
            let response = client
                .get(format!(
                    "{}/hkexwidget/data/getequityquote?sym={}&token={}&lang=chi&qid={}&callback=jQuery_{}&_={}",
                    url,
                    code,
                    token,
                    timestamp,
                    timestamp,
                    timestamp,
                ))
                .send()
                .await?;
            let text = response.text().await?;
            let json = remove_jquery_wrapping_fn_call(&text);
            let data = json.get("data").unwrap();
            let data = data.get("quote").unwrap();
            let v = data["vo"].as_str().unwrap().to_string();
            let vo_u = data["vo_u"].as_str().unwrap().to_string();
            let v = cal_value(&v, &vo_u);
            let am = data["am"].as_str().unwrap().to_string();
            let am_u = data["am_u"].as_str().unwrap().to_string();
            let am = cal_value(&am, &am_u);
            Ok(StockPriceDTO {
                h: data["hi"].as_str().unwrap().to_string(),
                l: data["lo"].as_str().unwrap().to_string(),
                o: data["op"].as_str().unwrap().to_string(),
                pc: data["pc"].as_str().unwrap().to_string(),
                p: data["ls"].as_str().unwrap().to_string(),
                cje: am.to_string(),
                ud: data["nc"].as_str().unwrap().to_string(),
                v: v.to_string(),
                yc: data["hc"].as_str().unwrap().to_string(),
                t: data["update_time"].as_str().unwrap().to_string(),
            })
        }
    }
}

fn cal_value(val: &str, unit: &str) -> BigDecimal {
    let val = BigDecimal::from_str(val).unwrap();
    let unit = match unit {
        "B" => BigDecimal::from(1000000000),
        "M" => BigDecimal::from(1000000),
        "K" => BigDecimal::from(1000),
        _ => BigDecimal::from(1),
    };
    val * unit
}

fn remove_jquery_wrapping_fn_call(data: &str) -> Value {
    // Remove the wrapping function call
    if let Some(start_idx) = data.find('(') {
        if let Some(end_idx) = data.rfind(')') {
            let json_str = &data[start_idx + 1..end_idx]; // Extract JSON string
                                                          // Parse the JSON string
            serde_json::from_str::<Value>(json_str).unwrap()
        } else {
            serde_json::from_str::<Value>(data).unwrap()
        }
    } else {
        serde_json::from_str::<Value>(data).unwrap()
    }
}
