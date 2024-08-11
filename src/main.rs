use std::error::Error;

use application::application::{Application, RustApplication};
use clap::Parser;
use cli::Cli;
use database::DbService;
use registration::{deregister, register};
use tokio::runtime::Runtime;
use web::bootstrap::Bootstrap;

use trading_bot::job::jobs::load_jobs;
use trading_bot::server::StockBotServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let application = RustApplication::default();

    application
        .publisher
        .subscribe(ApplicationEvent::Prepared, |application| {
            let db_service = Runtime::new()
                .unwrap()
                .block_on(DbService::create(&application.config));
            application.context.set(db_service);
        });

    application
        .publisher
        .subscribe(ApplicationEvent::ContextInited, |_application| {
            Runtime::new()
                .unwrap()
                .block_on(load_jobs())
                .expect("Fail to load jobs");
        });

    application.run()?;

    let cli: Cli = Cli::parse();

    let server = StockBotServer;
    server.run(&cli.command, register, deregister).await;

    Ok(())
}
