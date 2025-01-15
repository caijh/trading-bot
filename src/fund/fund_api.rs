use crate::exchange::exchange_model::Exchange;
use crate::stock::stock_model::Model as Stock;
use async_trait::async_trait;
use calamine::{open_workbook, Reader, Xlsx};
use rand::{thread_rng, Rng};
use serde_json::Value;
use std::error::Error;
use std::path::Path;
use tempfile::tempdir;
use util::request::Request;

#[async_trait]
pub trait FundApi {
    async fn get_funds(&self) -> Result<Vec<Stock>, Box<dyn Error>>;
}

#[async_trait]
impl FundApi for Exchange {
    async fn get_funds(&self) -> Result<Vec<Stock>, Box<dyn Error>> {
        match self {
            Exchange::SH => get_funds_from_sse(self.as_ref()).await,
            Exchange::SZ => {
                let url = format!("http://www.szse.cn/api/report/ShowReport?SHOWTYPE=xlsx&CATALOGID=1105&TABKEY=tab1&random={}", thread_rng().gen::<f64>());
                let dir = tempdir()?;
                let path_buf = dir.path().join("sz_funds.xlsx");
                Request::download(&url, path_buf.as_path()).await?;
                let stocks = read_funds_from_sz_excel(path_buf.as_path(), self.as_ref())?;
                Ok(stocks)
            }
            Exchange::HK => Ok(Vec::new()),
            Exchange::NASDAQ => Ok(Vec::new()),
        }
    }
}

async fn get_funds_from_sse(exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
    let url = format!(
        "https://query.sse.com.cn/commonSoaQuery.do?sqlId=FUND_LIST&fundType=00&_={}",
        thread_rng().gen::<f64>()
    );
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

fn read_funds_from_sz_excel(path: &Path, exchange: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
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
