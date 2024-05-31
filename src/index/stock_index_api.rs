use std::error::Error;
use std::str::FromStr;

use chrono::Local;
use configuration::Configuration;
use serde_json::Value;
use util::request::Request;

use crate::exchange::exchange_model::Exchange;
use crate::stock::stock_model::Stock;

pub async fn get_stocks(index: &str, exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
    let exchange = Exchange::from_str(exchange).unwrap();
    let client = Request::client().await;
    let mut stocks: Vec<Stock> = Vec::new();
    match exchange {
        Exchange::SH(exchange) => {
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
        Exchange::SZ(exchange) => {
            let config = Configuration::get_config().await;
            let url = config.get_string("stock.api.sz.baseurl").unwrap();
            let mut page_no = 1;
            loop {
                let url = format!("{}/api/report/ShowReport/data?SHOWTYPE=JSON&CATALOGID=1747_zs&PAGENO={}&ZSDM={}&random={}", url, page_no, index, Local::now().timestamp_millis());
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
