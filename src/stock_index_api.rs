use std::error::Error;
use std::str::FromStr;

use chrono::Local;
use serde_json::Value;
use util::request::Request;

use crate::exchange::Exchange;
use crate::stock::Stock;

pub async fn get_stocks(index: &str, exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
    let exchange = Exchange::from_str(exchange).unwrap();
    match exchange {
        Exchange::SH(exchange) => {
            let client = Request::client().await;
            let url = format!("https://query.sse.com.cn/commonSoaQuery.do?sqlId=DB_SZZSLB_CFGLB&indexCode={}&_={}", index, Local::now().timestamp_millis());
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".parse().unwrap());
            headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
            headers.insert("Referer", "https://www.sse.com.cn/".parse().unwrap());
            headers.insert("Connection", "keep-alive".parse().unwrap());
            let response = client.get(url).headers(headers).send().await;
            match response {
                Ok(response) => {
                    let json: Value = response.json().await?;
                    let result = json.get("result").unwrap().as_array();
                    let mut stocks: Vec<Stock> = Vec::new();
                    if let Some(ss) = result {
                        for s in ss {
                            let stock = Stock {
                                code: s["securityCode"].as_str().unwrap().to_string(),
                                name: s["securityAbbr"].as_str().unwrap().to_string(),
                                exchange: exchange.clone(),
                            };
                            stocks.push(stock);
                        }
                    }
                    Ok(stocks)
                }
                Err(e) => Err(e.into()),
            }
        }
        Exchange::SZ(_) => {
            let stocks: Vec<Stock> = Vec::new();
            Ok(stocks)
        }
    }
}
