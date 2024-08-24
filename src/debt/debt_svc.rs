use application::application::APPLICATION_CONTEXT;
use application::environment::Environment;
use chrono::{Local, NaiveDateTime};
use serde_json::Value;
use std::error::Error;
use util::request::Request;

use crate::debt::debt_model::DebtPrice;

pub async fn get_debt_price(code: &String) -> Result<DebtPrice, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.environment.read().await;
    let client = Request::client().await;
    let url = environment
        .get_property::<String>("stock.api.sh.baseurl")
        .unwrap();
    let response = client
        .get(format!(
            "{}/v1/shb1/snap/{}?_={}",
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
    Ok(DebtPrice {
        yc: snap.get(1).unwrap().to_string(),
        open: snap.get(2).unwrap().to_string(),
        high: snap.get(3).unwrap().to_string(),
        low: snap.get(4).unwrap().to_string(),
        current: snap.get(5).unwrap().to_string(),
        zd: snap.get(6).unwrap().to_string(),
        zdf: snap.get(7).unwrap().to_string(),
        v: snap.get(8).unwrap().to_string(),
        cje: snap.get(9).unwrap().to_string(),
        t: NaiveDateTime::parse_from_str(&format!("{}{}", date, time), "%Y%m%d%H%M%S")
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    })
}
