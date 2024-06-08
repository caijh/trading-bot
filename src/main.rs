use std::error::Error;

use clap::Parser;
use cli::Cli;
use registration::{deregister, register};
use web::bootstrap::Bootstrap;

use trading_bot::server::StockBotServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();

    let server = StockBotServer;
    server.run(&cli.command, register, deregister).await;

    Ok(())
}
