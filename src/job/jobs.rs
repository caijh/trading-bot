use application_beans::factory::bean_factory::{BeanFactory, ConfigurableBeanFactory};
use application_context::context::application_context::APPLICATION_CONTEXT;
use application_core::env::property_resolver::PropertyResolver;
use application_core::lang::runnable::Runnable;
use application_schedule::scheduler::Scheduler;
use async_trait::async_trait;
use database::DbService;
use notification::{Notification, NotificationConfig};
use std::error::Error;
use tokio::spawn;
use tracing::{error, info};

use crate::analysis::stock_analysis_ctrl::IndexAnalysisParams;
use crate::analysis::stock_analysis_model::AnalyzedStock;
use crate::analysis::stock_analysis_svc;
use crate::analysis::stock_analysis_svc::analysis_index;
use crate::holiday::holiday_svc::sync_holidays;
use crate::index::stock_index_model::{StockIndex, SyncIndexConstituents};
use crate::index::stock_index_svc::{
    get_all_stock_index, sync_constituent_stocks_daily_price, sync_constituents,
};
use crate::stock::stock_svc::sync;

pub async fn load_jobs() -> Result<(), Box<dyn Error>> {
    let scheduler = Scheduler::new().await?;

    let application_context = APPLICATION_CONTEXT.read().await;
    application_context.get_bean_factory().set(scheduler);
    let scheduler = application_context.get_bean_factory().get::<Scheduler>();
    scheduler.start().await?;

    Ok(())
}

pub struct SyncAllIndexStockPriceJob;

#[async_trait]
impl Runnable for SyncAllIndexStockPriceJob {
    async fn run(&self) {
        info!("SyncAllIndexStockPriceJob run ...");
        let indexes = get_all_stock_index().await.unwrap();
        for index in indexes {
            let _ = sync_constituent_stocks_daily_price(&index.code).await;
        }
        info!("SyncAllIndexStockPriceJob end success");
    }
}

pub struct AnalysisIndexStocksJob;

#[async_trait]
impl Runnable for AnalysisIndexStocksJob {
    async fn run(&self) {
        info!("AnalysisIndexStocksJob run ...");
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
                    spawn(notification_index_stocks_price(index, stocks));
                }
                Err(e) => {
                    error!("analysis index {} stocks fail, {}", index.name, e);
                }
            }
        }
        info!("AnalysisIndexStocksJob end success");
    }
}

pub struct AnalysisFundsJob;

#[async_trait]
impl Runnable for AnalysisFundsJob {
    async fn run(&self) {
        info!("AnalysisFundsJob run ...");
        let result = stock_analysis_svc::analysis_funds().await;
        match result {
            Ok(stocks) => {
                spawn(notification_stocks_price(
                    stocks,
                    StockIndex {
                        code: "".to_string(),
                        name: "基金".to_string(),
                        exchange: "".to_string(),
                    },
                ));
                info!("AnalysisFunsJob end successs");
            }
            Err(e) => {
                error!("analysis fund stocks fail, {}", e);
            }
        }
    }
}


async fn notification_index_stocks_price(index: StockIndex, stocks: Vec<AnalyzedStock>) {
    if stocks.is_empty() {
        return ;
    }

    // send max 5 stocks notification per request
    let mut stocks_to_send: Vec<AnalyzedStock> = Vec::new();
    for stock in stocks {        
        stocks_to_send.push(stock);
        if stocks_to_send.len() == 5 {
            let _ = notification_stocks_price(stocks_to_send.clone(), index.clone()).await;
            stocks_to_send.clear();
        }
    }
    if !stocks_to_send.is_empty() {
        notification_stocks_price(stocks_to_send.clone(), index).await;
    }
}


async fn notification_stocks_price(stocks: Vec<AnalyzedStock>, index: StockIndex) {
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

pub struct SyncIndexStocksJob;

#[async_trait]
impl Runnable for SyncIndexStocksJob {
    async fn run(&self) {
        info!("SyncIndexStocksJob run ...");
        let application_context = APPLICATION_CONTEXT.read().await;
        let dao = application_context
            .get_bean_factory()
            .get::<DbService>()
            .dao();
        let indexes = StockIndex::select_all(dao).await.unwrap();
        for index in indexes {
            let constituents = sync_constituents(&index.code).await.unwrap();
            spawn(notification_index_stocks_changed(index, constituents));
        }

        info!("SyncIndexStocksJob end success");
    }
}

async fn notification_index_stocks_changed(
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
pub struct SyncStocksJob {
    pub exchange: String,
}

#[async_trait]
impl Runnable for SyncStocksJob {
    async fn run(&self) {
        info!("SyncStocksJob run ...");
        let result = sync(&self.exchange).await;
        match result {
            Ok(_) => {
                info!("SyncStocksJob end success")
            }
            Err(e) => {
                error!("Sync {} stock error {}", &self.exchange, e);
            }
        }
    }
}

pub struct SyncHolidayJob;

#[async_trait]
impl Runnable for SyncHolidayJob {
    async fn run(&self) {
        let r = sync_holidays().await;
        match r {
            Ok(_) => {}
            Err(e) => {
                error!("sync holiday error {}", e)
            }
        }
    }
}
