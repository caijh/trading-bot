use calamine::{open_workbook, Reader, Xls};
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use tempfile::tempdir;
use tracing::info;

use crate::stock::stock_model::Stock;

pub async fn get_stocks(index: &str) -> Result<Vec<Stock>, Box<dyn Error>> {
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
pub async  fn read_index_stocks_from_excel(
    path: &Path,
) -> Result<Vec<Stock>, Box<dyn Error>> {
    let mut excel_xlsx: Xls<_> = open_workbook(path)?;

    let mut stocks = Vec::new();
    if let Some(result) = excel_xlsx.worksheet_range_at(0) {
        if let Ok(range) = result {
            for (i, row) in range.rows().into_iter().enumerate() {
                if i == 0 {
                    continue;
                }
                let exchange = if row[7] == "深圳证券交易所" { "SZ"} else {"SH"};
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
        let result = get_stocks("000016").await;
        assert!(result.is_ok());
    }
}
