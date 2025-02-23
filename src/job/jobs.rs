use application_beans::factory::bean_factory::{BeanFactory, ConfigurableBeanFactory};
use application_context::context::application_context::APPLICATION_CONTEXT;
use application_core::env::property_resolver::PropertyResolver;
use application_core::lang::runnable::Runnable;
use application_schedule::scheduler::Scheduler;
use async_trait::async_trait;
use database_mysql_seaorm::Dao;
use notification::{Notification, NotificationConfig};
use rand::{thread_rng, Rng};
use redis::Commands;
use redis_io::Redis;
use sea_orm::EntityTrait;
use std::error::Error;
use tokio::spawn;
use tracing::{error, info};

use crate::analysis::analysis_ctrl::IndexAnalysisParams;
use crate::analysis::analysis_model::AnalyzedStock;
use crate::analysis::analysis_svc;
use crate::analysis::analysis_svc::analysis_index;
use crate::fund::fund_svc;
use crate::index::index_constituent_model::SyncIndexConstituents;
use crate::index::index_svc::{
    get_all_stock_index, sync_constituent_stocks_daily_price, sync_constituents,
};
use crate::index::{index_constituent_model, index_model};
use crate::stock::stock_svc::{sync, sync_stock_daily_price};
use crate::token::token_svc;

use crate::index::index_model::Model as StockIndex;

pub async fn load_jobs() -> Result<(), Box<dyn Error>> {
    let scheduler = Scheduler::new().await?;

    let application_context = APPLICATION_CONTEXT.read().await;
    application_context.get_bean_factory().set(scheduler);
    let scheduler = application_context.get_bean_factory().get::<Scheduler>();
    scheduler.start().await?;

    let _ = scheduler
        .add_job(
            1,
            "同步HKEX的AccessToken",
            "0 0 9,12,15 * * *",
            Box::new(SyncHKEXTokenJob),
        )
        .await;
    // let _ = scheduler
    //     .add_job(
    //         2,
    //         "同步指数成分份股股价",
    //         "0 30 15,16 * * *",
    //         Box::new(SyncAllIndexStockPriceJob),
    //     )
    //     .await;

    // let _ = scheduler
    //     .add_job(
    //         3,
    //         "同步基金股价",
    //         "0 30 15,16 * * *",
    //         Box::new(SyncFundPriceJob),
    //     )
    //     .await;

    Ok(())
}

pub struct SyncAllIndexStockPriceJob;

#[async_trait]
impl Runnable for SyncAllIndexStockPriceJob {
    async fn run(&self) {
        let seconds = thread_rng().gen_range(1..10);
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;

        let client = Redis::get_client();
        let mut con = client.get_connection().unwrap();
        let key = "Sync:Index:Price".to_string();
        let value = con.get::<&str, Option<String>>(&key).unwrap();

        match value {
            None => {
                con.set_ex::<&str, &str, String>(&key, "doing", 3600)
                    .unwrap();
                info!("SyncAllIndexStockPriceJob run ...");
                let indexes = get_all_stock_index().await.unwrap();
                for index in indexes {
                    let _ = sync_constituent_stocks_daily_price(&index.code).await;
                }
                info!("SyncAllIndexStockPriceJob end success");
                let _ = con.del::<&str, i32>(&key);
            }
            Some(_value) => {
                info!("Job is running")
            }
        }
    }
}

pub struct SyncFundPriceJob;

#[async_trait]
impl Runnable for SyncFundPriceJob {
    async fn run(&self) {
        let seconds = thread_rng().gen_range(1..10);
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;

        let client = Redis::get_client();
        let mut con = client.get_connection().unwrap();
        let key = "Sync:Fund:Price".to_string();
        let value = con.get::<&str, Option<String>>(&key).unwrap();

        match value {
            None => {
                con.set_ex::<&str, &str, String>(&key, "doing", 3600)
                    .unwrap();
                info!("SyncFundPriceJob run ...");
                let funds = fund_svc::find_all().await.unwrap();
                for fund in funds {
                    let _ = sync_stock_daily_price(&fund.code).await;
                }
                info!("SyncFundPriceJob end success");
                let _ = con.del::<&str, i32>(&key);
            }
            Some(_value) => {
                info!("Job is running")
            }
        }
    }
}

pub struct AnalysisIndexStocksJob {
    pub code: Option<String>,
}

#[async_trait]
impl Runnable for AnalysisIndexStocksJob {
    async fn run(&self) {
        info!("AnalysisIndexStocksJob run ...");
        let application_context = APPLICATION_CONTEXT.read().await;
        let dao = application_context.get_bean_factory().get::<Dao>();
        let indexes = index_model::Entity::find()
            .all(&dao.connection)
            .await
            .unwrap();
        // filter indexes by code
        let indexes = match &self.code {
            Some(code) => indexes
                .into_iter()
                .filter(|index| &index.code == code)
                .collect(),
            None => indexes,
        };
        for index in indexes {
            let params = IndexAnalysisParams {
                code: Some(index.code.clone()),
            };
            let result = analysis_index(&params).await;
            match result {
                Ok(stocks) => {
                    spawn(notification_index_stocks_price(index, stocks));
                }
                Err(e) => {
                    error!("analysis index {} stocks fail, {}", index.name, e);
                    spawn(notification_error(format!(
                        "analysis index {} fail, {}",
                        index.name, e
                    )));
                }
            }
        }
        info!("AnalysisIndexStocksJob end success");
    }
}

pub struct AnalysisFundsJob {
    pub code: Option<String>,
}

#[async_trait]
impl Runnable for AnalysisFundsJob {
    async fn run(&self) {
        info!("AnalysisFundsJob run ...");
        let code = self.code.clone();
        let result = analysis_svc::analysis_funds(code).await;
        match result {
            Ok(stocks) => {
                spawn(notification_index_stocks_price(
                    StockIndex {
                        code: "".to_string(),
                        name: "基金".to_string(),
                        exchange: "".to_string(),
                    },
                    stocks,
                ));
                info!("AnalysisFundsJob end success");
            }
            Err(e) => {
                error!("analysis fund stocks fail, {}", e);
                spawn(notification_error(format!("analysis fund fail, {}", e)));
            }
        }
    }
}

async fn notification_index_stocks_price(index: StockIndex, stocks: Vec<AnalyzedStock>) {
    if stocks.is_empty() {
        return;
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
        let patterns = stock.pattern.join(",");
        content.push_str(
            format!(
                "{:<5} {} C: {} S: {} R: {} {}\n",
                stock.name, stock.code, stock.current, stock.support, stock.resistance, patterns
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
                "{}/send/user/{}",
                notification_config.url, notification_config.receiver
            );
            Notification::create(&title, &content)
                .send(url.as_str(), notification_config.receiver.as_str())
                .await
        }
    }
}

async fn notification_error(content: String) {
    if content.is_empty() {
        return;
    }
    let title = "错误提醒-".to_string();
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.get_environment().await;
    let result = environment.get_property::<NotificationConfig>("notification");
    match result {
        None => {}
        Some(notification_config) => {
            let url = format!(
                "{}/send/user/{}",
                notification_config.url, notification_config.receiver
            );
            Notification::create(&title, &content)
                .send(url.as_str(), notification_config.receiver.as_str())
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
        let dao = application_context.get_bean_factory().get::<Dao>();
        let indexes = index_model::Entity::find()
            .all(&dao.connection)
            .await
            .unwrap();
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
    let mut stocks_to_send = Vec::new();
    for stock in stocks_add {
        stocks_to_send.push(stock);
        if stocks_to_send.len() == 10 {
            let _ =
                do_notification_index_stocks_changed(&index, stocks_to_send.clone(), true).await;
            stocks_to_send.clear();
        }
    }
    if !stocks_to_send.is_empty() {
        do_notification_index_stocks_changed(&index, stocks_to_send.clone(), true).await;
    }
    for stock in stocks_remove {
        stocks_to_send.push(stock);
        if stocks_to_send.len() == 10 {
            let _ =
                do_notification_index_stocks_changed(&index, stocks_to_send.clone(), false).await;
            stocks_to_send.clear();
        }
    }
    if !stocks_to_send.is_empty() {
        do_notification_index_stocks_changed(&index, stocks_to_send.clone(), false).await;
    }
}

async fn do_notification_index_stocks_changed(
    index: &StockIndex,
    index_constituents: Vec<index_constituent_model::Model>,
    add: bool,
) {
    if index_constituents.is_empty() {
        return;
    }

    let title = "指数成分股关注-".to_string() + index.name.as_str();
    let mut content = "".to_string();
    let label = if add { "增加" } else { "移除" };
    for stock in index_constituents {
        content
            .push_str(format!("{} {:<5} {}\n", label, stock.stock_name, stock.stock_code).as_str());
    }
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.get_environment().await;
    let result = environment.get_property::<NotificationConfig>("notification");
    match result {
        None => {}
        Some(notification_config) => {
            let url = format!(
                "{}/send/user/{}",
                notification_config.url, notification_config.receiver
            );
            Notification::create(&title, &content)
                .send(url.as_str(), notification_config.receiver.as_str())
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

pub struct SyncHKEXTokenJob;
#[async_trait]
impl Runnable for SyncHKEXTokenJob {
    async fn run(&self) {
        let r = token_svc::reset_hkex_token().await;
        match r {
            Ok(_) => {
                info!("Sync HKEX token success");
            }
            Err(e) => {
                error!("Sync HKEX token error {}", e)
            }
        }
    }
}
