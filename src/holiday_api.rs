use std::error::Error;

use util::request::Request;

pub async fn get_holidays() -> Result<Vec<String>, Box<dyn Error>> {
    let url =
        "https://raw.githubusercontent.com/rainx/cn_stock_holidays/main/cn_stock_holidays/data.txt";
    let response = Request::get_response(url).await?;
    let content = response.text().await?;
    let dates: Vec<String> = content
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    Ok(dates)
}
