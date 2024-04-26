use std::error::Error;

use chrono::{Local, NaiveDateTime};
use configuration::Configuration;
use context::SERVICES;
use database::DbService;
use rand::seq::SliceRandom;
use rbatis::rbatis_codegen::ops::AsProxy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use util::request::Request;

use crate::stock::Stock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockDTO {
    pub dm: String,
    pub mc: String,
    pub jys: String,
}

/// split licence, get random licence from it
fn get_licence(licence: String) -> String {
    let mut licences = licence.split(',').collect::<Vec<&str>>();

    let mut rng = rand::thread_rng();
    licences.shuffle(&mut rng);
    match licences.first() {
        Some(licence) => licence.to_string(),
        None => licence,
    }
}

pub async fn get_stocks() -> Result<Vec<StockDTO>, Box<dyn Error>> {
    let client = Request::client().await;
    let config = Configuration::get_config().await;
    let url = config.get_string("stock.api.biying.baseurl").unwrap();
    let licence = get_licence(config.get_string("stock.api.biying.licence").unwrap());
    let response = client
        .get(format!("{}/hslt/list/{}", url, licence))
        .send()
        .await?;
    let stocks: Vec<StockDTO> = response.json().await.unwrap();
    Ok(stocks)
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
    pub zf: String,
    pub hs: String,
    pub zd: String,
    pub zde: String,
}

pub async fn get_stock_daily_price(code: &str) -> Result<Vec<StockDailyPriceDTO>, Box<dyn Error>> {
    let client = Request::client().await;
    let config = Configuration::get_config().await;
    let url = config.get_string("stock.api.biying.baseurl").unwrap();
    let licence = get_licence(config.get_string("stock.api.biying.licence").unwrap());
    let response = client
        .get(format!("{}/hszbl/fsjy/{}/dh/{}", url, code, licence))
        .send()
        .await?;
    let stocks: Vec<StockDailyPriceDTO> = response.json().await.unwrap();
    Ok(stocks)
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
    if stock.is_none() {
        return Err("stock not found".into());
    }
    let stock = stock.unwrap();
    let client = Request::client().await;
    let config = Configuration::get_config().await;
    if "hz" == stock.exchange {
        let url = config.get_string("stock.api.hz.baseurl").unwrap();
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
        let snp = json.get("snap").unwrap();
        let date = json.get("date").unwrap().to_string();
        let time = json.get("time").unwrap().to_string();
        Ok(StockPriceDTO {
            h: snp.get(3).unwrap().to_string().string(),
            l: snp.get(4).unwrap().to_string(),
            o: snp.get(2).unwrap().to_string(),
            pc: snp.get(7).unwrap().to_string(),
            p: snp.get(5).unwrap().to_string(),
            cje: snp.get(10).unwrap().to_string(),
            ud: snp.get(8).unwrap().to_string(),
            v: snp.get(9).unwrap().to_string(),
            yc: snp.get(1).unwrap().to_string(),
            t: NaiveDateTime::parse_from_str((date + &time).as_str(), "%Y%m%d%H%M%S").unwrap().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    } else {
        let url = config.get_string("stock.api.sz.baseurl").unwrap();
        let response = client
            .get(format!(
                "{}/api/market/ssjjhq/getTimeData?random={}&marketId=1&code={}",
                url,
                Local::now().timestamp_millis(),
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
