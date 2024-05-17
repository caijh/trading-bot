use std::error::Error;

use clap::Parser;
use cli::Cli;
use configuration::Configuration;
use registration::deregister;
use web::bootstrap::Bootstrap;

use stock_bot::server::StockBotServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();

    let server = StockBotServer;
    server
        .run(
            &cli.command,
            || {
                tokio::spawn(register());
            },
            || {
                tokio::spawn(deregister());
            },
        )
        .await;

    Ok(())
}

pub async fn register() {
    let config = Configuration::get_config().await;
    let _ = registration::register(&config).await;
}
