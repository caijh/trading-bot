use crate::exchange::{exchange_model, market_time};
use crate::stock::stock_svc;
use application_beans::factory::bean_factory::BeanFactory;
use application_context::context::application_context::APPLICATION_CONTEXT;
use chrono::Utc;
use database_mysql_seaorm::Dao;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::error::Error;
use std::str::FromStr;

pub async fn get_market_status_by_stock_code(code: &str) -> Result<String, Box<dyn Error>> {
    let stock = stock_svc::get_stock(code).await?;
    let exchange = &stock.exchange;
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();
    let market_times = market_time::Entity::find()
        .filter(market_time::Column::Exchange.eq(exchange))
        .order_by_asc(market_time::Column::StartTime)
        .all(&dao.connection)
        .await?;
    if market_times.is_empty() {
        return Ok("MarketTrading".to_string());
    }

    let exchange = exchange_model::Exchange::from_str(exchange)?;
    let tz = exchange.time_zone();
    let time = Utc::now().with_timezone(&tz).time();
    let first = market_times.first().unwrap();
    if time < first.start_time {
        return Ok("MarketClosed".to_string());
    }
    let last = market_times.last().unwrap();
    if time > last.end_time {
        return Ok("MarketClosed".to_string());
    }
    for market_time in market_times {
        if market_time.start_time <= time && time <= market_time.end_time {
            return Ok("MarketTrading".to_string());
        }
    }

    Ok("MarketClosed".to_string())
}
