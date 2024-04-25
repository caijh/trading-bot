use std::error::Error;

use context::SERVICES;
use database::DbService;
use rbatis::rbdc::Decimal;

use crate::stock::{Stock, StockDailyPrice, StockDailyPriceSyncRecord, StockPrice};
use crate::{stock, stock_api};

pub async fn init_stocks() -> Result<(), Box<dyn Error>> {
    let stocks = stock_api::get_stocks().await?;

    let db = SERVICES.get::<DbService>().dao();
    for stock_dto in stocks {
        let stock = Stock::select_by_code(db, &stock_dto.dm).await?;
        let s = stock_dto.clone();
        if stock.is_none() {
            Stock::insert(
                db,
                &Stock {
                    code: s.dm,
                    name: s.mc,
                    exchange: s.jys,
                },
            )
            .await?;
        } else {
            Stock::update_by_column(
                db,
                &Stock {
                    code: s.dm,
                    name: s.mc,
                    exchange: s.jys,
                },
                stock::COLUMN_CODE,
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn get_stock_daily_price(code: &str) -> Result<Vec<StockDailyPrice>, Box<dyn Error>> {
    let db = SERVICES.get::<DbService>().dao();
    let date = chrono::Local::now()
        .format("%Y%m%d")
        .to_string()
        .parse::<i64>()
        .unwrap();
    let mut daily_prices: Vec<StockDailyPrice> =
        StockDailyPrice::select_by_column(db, "code", code).await?;
    let mut updated: bool = false;
    if let Some(stock_daily_price_sync_record) =
        StockDailyPriceSyncRecord::select_by_code_date(db, code, date).await?
    {
        updated = stock_daily_price_sync_record.updated;
    }
    if !updated {
        let daily_price_dtos = stock_api::get_stock_daily_price(code).await?;
        let dates: Vec<i64> = daily_prices.iter().map(|e| e.date).collect();
        for dto in daily_price_dtos {
            let daily_price = StockDailyPrice {
                code: code.to_string(),
                date: dto.d.replace('-', "").parse::<i64>().unwrap(),
                open: Some(Decimal::new(&dto.o).unwrap()),
                close: Some(Decimal::new(&dto.c).unwrap()),
                high: Some(Decimal::new(&dto.h).unwrap()),
                low: Some(Decimal::new(&dto.l).unwrap()),
                volume: Some(Decimal::new(&dto.v).unwrap()),
                amount: Some(Decimal::new(&dto.e).unwrap()),
                zf: Some(Decimal::new(&dto.zf).unwrap()),
                hs: Some(Decimal::new(&dto.hs).unwrap()),
                zd: Some(Decimal::new(&dto.zd).unwrap()),
                zde: Some(Decimal::new(&dto.zde).unwrap()),
            };
            if !dates.contains(&daily_price.date) {
                StockDailyPrice::insert(db, &daily_price).await?;
                daily_prices.push(daily_price);
            }
        }
        StockDailyPriceSyncRecord::insert(
            db,
            &StockDailyPriceSyncRecord {
                code: code.to_string(),
                date,
                updated: true,
            },
        )
        .await?;
    }

    Ok(daily_prices)
}

pub async fn get_stock_price(code: &str) -> Result<StockPrice, Box<dyn Error>> {
    let price_dto = stock_api::get_current_price(code).await?;

    let price = StockPrice {
        code: code.to_string(),
        high: Some(Decimal::new(&price_dto.h).unwrap()),
        low: Some(Decimal::new(&price_dto.l).unwrap()),
        open: Some(Decimal::new(&price_dto.o).unwrap()),
        pc: Some(Decimal::new(&price_dto.pc).unwrap()),
        price: Some(Decimal::new(&price_dto.p).unwrap()),
        amount: Some(Decimal::new(&price_dto.cje).unwrap()),
        ud: Some(Decimal::new(&price_dto.ud).unwrap()),
        volume: Some(Decimal::new(&price_dto.v).unwrap()),
        yc: None,
        zf: None,
        zs: None,
        time: price_dto.t.clone(),
    };

    Ok(price)
}
