use application_boot::application::APPLICATION_CONTEXT;
use application_core::env::property_resolver::PropertyResolver;
use chrono::Local;
use rand::{thread_rng, Rng};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::error::Error;
use std::str::FromStr;
use tracing::info;
use util::request::Request;

use crate::exchange::exchange_model::Exchange;
use crate::stock::stock_model::Stock;

pub async fn get_stocks(index: &str, exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
    let exchange = Exchange::from_str(exchange)?;
    let client = Request::client().await;
    let mut stocks: Vec<Stock> = Vec::new();
    let application_context = APPLICATION_CONTEXT.read().await;
    match exchange {
        Exchange::SH(_exchange) => {
            let result = get_sh_index_stocks(index).await;
            match result {
                Ok(index_stocks) => {
                    for s in index_stocks {
                        stocks.push(s);
                    }
                    Ok(stocks)
                }
                Err(e) => Err(e.into()),
            }
        }
        Exchange::SZ(exchange) => {
            let environment = application_context.get_environment().await;
            let url = environment
                .get_property::<String>("stock.api.sz.baseurl")
                .unwrap();
            let mut page_no = 1;
            loop {
                let url = format!("{}/api/report/ShowReport/data?SHOWTYPE=JSON&CATALOGID=1747_zs&PAGENO={}&ZSDM={}&random={}", url, page_no, index, thread_rng().gen::<f64>());
                info!("Exchange sz query stocks url = {}", url);
                let response = client.get(url).send().await;
                page_no += 1;
                match response {
                    Ok(response) => {
                        let json: Value = response.json().await?;
                        let data = json.get(0).unwrap();
                        let result = data.get("data").unwrap().as_array();
                        if result.is_none() {
                            break;
                        }
                        if let Some(ss) = result {
                            if ss.is_empty() {
                                break;
                            }
                            for s in ss {
                                let stock = Stock {
                                    code: s["zqdm"].as_str().unwrap().to_string(),
                                    name: s["zqjc"]
                                        .as_str()
                                        .unwrap()
                                        .to_string()
                                        .replace("&nbsp;", " "),
                                    exchange: exchange.clone(),
                                    stock_type: "Stock".to_string(),
                                    to_code: None,
                                };
                                stocks.push(stock);
                            }
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            Ok(stocks)
        }
    }
}

async fn get_sh_index_stocks(index: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
    let url = format!(
        "https://query.sse.com.cn/commonSoaQuery.do?sqlId=DB_SZZSLB_CFGLB&indexCode={}&_={}",
        index,
        Local::now().timestamp_millis()
    );
    info!("Exchange sh query stocks url = {}", url);

    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
    headers.insert("Referer", "https://www.sse.com.cn/".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    let client = Request::client().await;
    let response = client.get(url).headers(headers).send().await;
    let mut stocks: Vec<Stock> = Vec::new();
    match response {
        Ok(response) => {
            let json: Value = response.json().await?;
            let result = json.get("result").unwrap().as_array();
            if let Some(ss) = result {
                for s in ss {
                    let stock = Stock {
                        code: s["securityCode"].as_str().unwrap().to_string(),
                        name: s["securityAbbr"].as_str().unwrap().to_string(),
                        exchange: "SH".to_string(),
                        stock_type: "Stock".to_string(),
                        to_code: None,
                    };
                    stocks.push(stock);
                }
            }
            Ok(stocks)
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_sh_index_stocks() {
        let result = get_sh_index_stocks("000016").await;
        assert!(result.is_ok());
    }
}
