use std::error::Error;

use configuration::Configuration;
use serde::{Deserialize, Serialize};
use util::request::Request;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockDTO {
    pub dm: String,
    pub mc: String,
    pub jys: String,
}

pub async fn get_stocks() -> Result<Vec<StockDTO>, Box<dyn Error>> {
    let client = Request::client().await;
    let config = Configuration::get_config().await;
    let url = config.get_string("stock.base_url").unwrap();
    let licence = config.get_string("stock.licence").unwrap();
    let response = client.get(format!("{}/hslt/list/{}", url, licence)).send().await?;
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
    let url = config.get_string("stock.base_url").unwrap();
    let licence = config.get_string("stock.licence").unwrap();
    let response = client.get(format!("{}/hszbl/fsjy/{}/dh/{}", url, code, licence)).send().await?;
    let stocks: Vec<StockDailyPriceDTO> = response.json().await.unwrap();
    Ok(stocks)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockPriceDTO {
    pub fm: String,
    pub h: String,
    pub hs: String,
    pub lb: String,
    pub l: String,
    pub  lt: String,
    pub  o: String,
    pub  pe: String,
    pub  pc: String,
    pub  p: String,
    pub  sz: String,
    pub  cje: String,
    pub  ud: String,
    pub  v: String,
    pub  yc: String,
    pub  zf: String,
    pub  zs: String,
    pub  sjl: String,
    pub  zdf60: String,
    pub  zdfnc: String,
    pub t: String,
}

pub async fn get_current_price(code: &str)-> Result<StockPriceDTO, Box<dyn Error>> {
    let client = Request::client().await;
    let config = Configuration::get_config().await;
    let url = config.get_string("stock.base_url").unwrap();
    let licence = config.get_string("stock.licence").unwrap();
    let response = client.get(format!("{}/hsrl/ssjy/{}/{}", url, code, licence)).send().await?;
    let price: StockPriceDTO = response.json().await.unwrap();
    Ok(price)
}

