use crate::exchange::exchange_model::Exchange;
use crate::stock::stock_model::Model as Stock;
use calamine::{open_workbook, Reader, Xls};
use rand::Rng;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use tempfile::tempdir;
use tracing::info;
use util::request::Request;

pub async fn get_stocks(exchange: &Exchange, index: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
    match exchange {
        Exchange::SH | Exchange::SZ => {
            let url = format!(
                "https://csi-web-dev.oss-cn-shanghai-finance-1-pub.aliyuncs.com/static/html/csindex/public/uploads/file/autofile/cons/{}cons.xls",
                index,
            );
            info!("Query Index Stocks from url = {}", url);
            let dir = tempdir()?;
            let path = dir.path().join(format!("{}cons.xls", index));
            download(&url, &path).await?;
            let stocks = read_index_stocks_from_excel(&path).await?;
            Ok(stocks)
        }
        Exchange::HK => {
            let url = format!(
                "https://www.hsi.com.hk/data/schi/rt/index-series/{}/constituents.do?{}",
                index,
                rand::thread_rng().gen_range(1000..9999)
            );
            info!("Query Index Stocks from url = {}", url);
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
    }
}

async fn download(url: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::builder().build().unwrap();
    let response = client.get(url).send().await;
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
pub async fn read_index_stocks_from_excel(path: &Path) -> Result<Vec<Stock>, Box<dyn Error>> {
    let mut excel_xlsx: Xls<_> = open_workbook(path)?;

    let mut stocks = Vec::new();
    if let Some(result) = excel_xlsx.worksheet_range_at(0) {
        if let Ok(range) = result {
            for (i, row) in range.rows().into_iter().enumerate() {
                if i == 0 {
                    continue;
                }
                let exchange = if row[7] == "深圳证券交易所" {
                    "SZ"
                } else {
                    "SH"
                };
                stocks.push(Stock {
                    code: row[4].to_string(),
                    name: row[5].to_string(),
                    exchange: exchange.to_string(),
                    stock_type: "Stock".to_string(),
                    to_code: None,
                });
            }
        }
    }

    Ok(stocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_sh_index_stocks() {
        let result = get_stocks(&Exchange::SH, "000016").await;
        assert!(result.is_ok());
    }
}
