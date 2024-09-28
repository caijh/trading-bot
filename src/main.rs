use application_boot::application::{Application, RustApplication};
use std::error::Error;
use trading_bot::initializer::RoutInitializer;
use trading_bot::listener::{
    ApplicationContextInitializedListener, ApplicationStartedEventListener,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let application = RustApplication::default();

    application
        .add_listener(Box::new(ApplicationContextInitializedListener {}))
        .await;
    application
        .add_listener(Box::new(ApplicationStartedEventListener {}))
        .await;
    application
        .add_servlet_context_initializer(Box::new(RoutInitializer {}))
        .await;
    application.run().await?;

    Ok(())
}
