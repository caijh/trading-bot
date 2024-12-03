use application_boot::application::{Application, RustApplication};
use std::error::Error;
use trading_bot::listener::{
    ApplicationContextInitializedListener, ApplicationStartedEventListener,
};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn Error>> {
    let application = RustApplication::default();

    application
        .add_listener(Box::new(ApplicationContextInitializedListener {}))
        .await;
    application
        .add_listener(Box::new(ApplicationStartedEventListener {}))
        .await;
    application.run().await?;

    Ok(())
}
