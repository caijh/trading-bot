use std::error::Error;

use anyhow::Result;
use configuration::Configuration;
use context::SERVICES;
use database::DbService;
use notification::{Notification, NotificationConfig};
use tokio::spawn;
use tokio_cron_scheduler::{JobBuilder, JobScheduler};
use tracing::{error, info};

use crate::analysis::stock_analysis_ctrl::Params;
use crate::analysis::stock_analysis_svc::analysis;
use crate::exchange::exchange_model::Exchange;
use crate::holiday::holiday_svc::sync_holidays;
use crate::index::stock_index::{IndexConstituent, StockIndex};
use crate::index::stock_index_svc::{
    get_all_stock_index, get_constituent_stocks, sync_constituent_stocks_daily_price,
    sync_constituents,
};
use crate::stock::stock_svc::{get_stock_daily_price, sync_stock_daily_price, sync_stocks};

pub async fn load_jobs() -> Result<()> {
    let scheduler = create_scheduler().await?;
    scheduler.start().await?;

    add_sync_holidays_job(&scheduler).await?;

    add_sync_stocks_job(&scheduler).await?;

    add_sync_index_stocks_job(&scheduler).await?;

    add_sync_stock_price_job(&scheduler).await?;

    add_analysis_stocks_job(&scheduler).await?;

    Ok(())
}

async fn add_sync_stock_price_job(scheduler: &JobScheduler) -> Result<()> {
    let job = JobBuilder::new()
        .with_timezone(chrono_tz::Asia::Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 16 * * * *")
        .unwrap()
        .with_run_async(Box::new(|_uuid, _locked| {
            Box::pin(async move {
                let indexes = get_all_stock_index().await;
                match indexes {
                    Ok(indexes) => {
                        for index in indexes {
                            let _ = sync_constituent_stocks_daily_price(&index.code).await;
                        }
                    }
                    Err(_) => {}
                }
            })
        }))
        .build()?;
    scheduler.add(job).await?;
    Ok(())
}

async fn add_analysis_stocks_job(scheduler: &JobScheduler) -> Result<()> {
    let jj = JobBuilder::new()
        .with_timezone(chrono_tz::Asia::Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 18 * * Mon-Fri *")
        .unwrap()
        .with_run_async(Box::new(|_uuid, _locked| {
            Box::pin(async move {
                let dao = SERVICES.get::<DbService>().dao();
                let indexes = StockIndex::select_all(dao).await.unwrap();
                for index in indexes {
                    let params = Params {
                        index_code: index.code.clone(),
                    };
                    let result = analysis(&params).await;
                    match result {
                        Ok(stocks) => {
                            spawn(notification_stocks(stocks, index));
                        }
                        Err(e) => {
                            error!("analysis index {} stocks fail, {}", index.name, e);
                        }
                    }
                }
            })
        }))
        .build()?;
    scheduler.add(jj).await?;
    Ok(())
}

async fn notification_stocks(stocks: Vec<IndexConstituent>, index: StockIndex) {
    if stocks.is_empty() {
        return;
    }
    let title = "股票关注-".to_string() + index.name.as_str();
    let mut content = "".to_string();
    for stock in stocks {
        content.push_str(format!("{} {}\n", stock.stock_name, stock.stock_code).as_str());
    }
    let config = Configuration::get_config().await;
    let result = config.get::<NotificationConfig>("notification");
    match result {
        Ok(notification_config) => {
            let url = format!(
                "{}/send/{}",
                notification_config.url, notification_config.receiver
            );
            Notification::create(&title, &content)
                .send(
                    url.as_str(),
                    notification_config.token.as_str(),
                    notification_config.receiver.as_str(),
                )
                .await
        }
        Err(e) => {
            tracing::debug!("{:?}", e);
        }
    }
}

async fn add_sync_index_stocks_job(scheduler: &JobScheduler) -> Result<()> {
    let jj = JobBuilder::new()
        .with_timezone(chrono_tz::Asia::Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 1 * * * *")
        .unwrap()
        .with_run_async(Box::new(|_uuid, _locked| {
            Box::pin(async move {
                let dao = SERVICES.get::<DbService>().dao();
                let indexes = StockIndex::select_all(dao).await.unwrap();
                for index in indexes {
                    let _ = sync_constituents(&index.code).await;
                }
            })
        }))
        .build()?;
    scheduler.add(jj).await?;
    Ok(())
}

async fn add_sync_stocks_job(scheduler: &JobScheduler) -> Result<()> {
    let jj = JobBuilder::new()
        .with_timezone(chrono_tz::Asia::Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 0 * * * *")
        .unwrap()
        .with_run_async(Box::new(|_uuid, _locked| {
            Box::pin(async move {
                let _ = sync_stocks(&Exchange::SH("SH".to_string())).await;
                let _ = sync_stocks(&Exchange::SZ("SZ".to_string())).await;
            })
        }))
        .build()?;
    scheduler.add(jj).await?;
    Ok(())
}

async fn add_sync_holidays_job(scheduler: &JobScheduler) -> Result<()> {
    let jj = JobBuilder::new()
        .with_timezone(chrono_tz::Asia::Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 0 1 * * *")
        .unwrap()
        .with_run_async(Box::new(|_uuid, _locked| {
            Box::pin(async move {
                let _ = sync_holidays().await;
            })
        }))
        .build()?;
    scheduler.add(jj).await?;
    Ok(())
}

async fn create_scheduler() -> Result<JobScheduler> {
    let mut scheduler = JobScheduler::new().await?;

    #[cfg(feature = "signal")]
    scheduler.shutdown_on_ctrl_c();

    scheduler.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            info!("Shut down done");
        })
    }));

    Ok(scheduler)
}
