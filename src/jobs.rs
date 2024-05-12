use anyhow::Result;
use context::SERVICES;
use database::DbService;
use tokio_cron_scheduler::{JobBuilder, JobScheduler};

use crate::exchange::Exchange;
use crate::holiday_svc::sync_holidays;
use crate::stock_index::StockIndex;
use crate::stock_index_svc::sync_constituents;
use crate::stock_svc::sync_stocks;

pub async fn load_jobs() -> Result<()> {
    let scheduler = create_scheduler().await?;
    scheduler.start().await?;

    add_sync_holidays_job(&scheduler).await?;

    add_sync_stocks_job(&scheduler).await?;

    add_sync_index_stocks_job(&scheduler).await?;

    Ok(())
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
            println!("Shut down done");
        })
    }));

    Ok(scheduler)
}
