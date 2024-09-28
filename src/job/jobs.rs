use anyhow::Result;
use application_beans::factory::bean_factory::BeanFactory;
use application_boot::application::APPLICATION_CONTEXT;
use application_core::env::property_resolver::PropertyResolver;
use chrono::Local;
use database::DbService;
use notification::{Notification, NotificationConfig};
use tokio::spawn;
use tokio_cron_scheduler::{JobBuilder, JobScheduler};
use tracing::{error, info};

use crate::analysis::stock_analysis_ctrl::IndexAnalysisParams;
use crate::analysis::stock_analysis_model::AnalyzedStock;
use crate::analysis::stock_analysis_svc;
use crate::analysis::stock_analysis_svc::analysis_index;
use crate::holiday::holiday_svc::{is_holiday, sync_holidays, today_is_holiday};
use crate::index::stock_index_model::{StockIndex, SyncIndexConstituents};
use crate::index::stock_index_svc::{
    get_all_stock_index, sync_constituent_stocks_daily_price, sync_constituents,
};
use crate::stock::stock_svc::sync;

pub async fn load_jobs() -> Result<()> {
    let scheduler = create_scheduler().await?;
    scheduler.start().await?;

    // 同步节假日
    add_sync_holidays_job(&scheduler).await?;

    // 同步交易所股票
    add_sync_stocks_job(&scheduler).await?;

    // 同步指数股票
    add_sync_index_stocks_job(&scheduler).await?;

    // 同步指数股票价格
    add_sync_stock_price_job(&scheduler).await?;

    // 分析指数股票
    add_analysis_stocks_job(&scheduler).await?;

    // 分析基金
    add_analysis_funds_job(&scheduler).await?;

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
                let indexes = get_all_stock_index().await.unwrap();
                for index in indexes {
                    let _ = sync_constituent_stocks_daily_price(&index.code).await;
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
        .with_schedule("0 0 17 * * Mon-Fri *")
        .unwrap()
        .with_run_async(Box::new(|_uuid, _locked| {
            Box::pin(async move {
                let now = Local::now();
                let holiday_result = is_holiday(&now).await.unwrap();
                if holiday_result.is_holiday {
                    return;
                }
                let application_context = APPLICATION_CONTEXT.read().await;
                let dao = application_context
                    .get_bean_factory()
                    .get::<DbService>()
                    .dao();
                let indexes = StockIndex::select_all(dao).await.unwrap();
                for index in indexes {
                    let params = IndexAnalysisParams {
                        code: index.code.clone(),
                    };
                    let result = analysis_index(&params).await;
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

async fn add_analysis_funds_job(scheduler: &JobScheduler) -> Result<()> {
    let jj = JobBuilder::new()
        .with_timezone(chrono_tz::Asia::Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 18 * * Mon-Fri *")
        .unwrap()
        .with_run_async(Box::new(|_uuid, _locked| {
            Box::pin(async move {
                let holiday_result = today_is_holiday().await.unwrap();
                if holiday_result.is_holiday {
                    return;
                }
                let result = stock_analysis_svc::analysis_funds().await;
                match result {
                    Ok(stocks) => {
                        spawn(notification_stocks(
                            stocks,
                            StockIndex {
                                code: "".to_string(),
                                name: "基金".to_string(),
                                exchange: "".to_string(),
                            },
                        ));
                    }
                    Err(e) => {
                        error!("analysis fund stocks fail, {}", e);
                    }
                }
            })
        }))
        .build()?;
    scheduler.add(jj).await?;
    Ok(())
}

async fn notification_stocks(stocks: Vec<AnalyzedStock>, index: StockIndex) {
    if stocks.is_empty() {
        return;
    }
    let title = "股票关注-".to_string() + index.name.as_str();
    let mut content = "".to_string();
    for stock in stocks {
        content.push_str(
            format!(
                "{:<5} {} {}, C: {}, MIN20: {}, MAX20: {}\n",
                stock.name, stock.code, stock.pattern, stock.current, stock.min, stock.max,
            )
            .as_str(),
        );
    }
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.get_environment().await;
    let result = environment.get_property::<NotificationConfig>("notification");
    match result {
        None => {}
        Some(notification_config) => {
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
                let application_context = APPLICATION_CONTEXT.read().await;
                let dao = application_context
                    .get_bean_factory()
                    .get::<DbService>()
                    .dao();
                let indexes = StockIndex::select_all(dao).await.unwrap();
                for index in indexes {
                    let constituents = sync_constituents(&index.code).await.unwrap();
                    spawn(notification_index_stocks(index, constituents));
                }
            })
        }))
        .build()?;
    scheduler.add(jj).await?;
    Ok(())
}

async fn notification_index_stocks(
    index: StockIndex,
    sync_index_constituents: SyncIndexConstituents,
) {
    let stocks_add = sync_index_constituents.added;
    let stocks_remove = sync_index_constituents.removed;
    if stocks_add.is_empty() && stocks_remove.is_empty() {
        return;
    }

    let title = "指数成分股关注-".to_string() + index.name.as_str();
    let mut content = "".to_string();
    for stock in stocks_add {
        content.push_str(format!("增加 {:<5} {}\n", stock.stock_name, stock.stock_code).as_str());
    }
    for stock in stocks_remove {
        content.push_str(format!("移除 {:<5} {}\n", stock.stock_name, stock.stock_code).as_str());
    }
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.get_environment().await;
    let result = environment.get_property::<NotificationConfig>("notification");
    match result {
        None => {}
        Some(notification_config) => {
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
    }
}

async fn add_sync_stocks_job(scheduler: &JobScheduler) -> Result<()> {
    let jj = JobBuilder::new()
        .with_timezone(chrono_tz::Asia::Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 0 * * * *")
        .unwrap()
        .with_run_async(Box::new(|_uuid, _locked| {
            Box::pin(async move {
                let _ = sync("SH").await;
                let _ = sync("SZ").await;
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

    #[cfg(feature = "signal")]
    scheduler.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            info!("Scheduler Shutdown done");
        })
    }));

    Ok(scheduler)
}
