use std::error::Error;

use application::application::{Application, RustApplication};
use trading_bot::initializer::RoutInitializer;
use trading_bot::listener::{
    ApplicationContextInitializedListener, ApplicationStartedEventListener,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut application = RustApplication::default();

    application.add_listener(Box::new(ApplicationContextInitializedListener {}));
    application.add_listener(Box::new(ApplicationStartedEventListener {}));
    application.add_servlet_context_initializer(Box::new(RoutInitializer {}));
    application.run()?;

    Ok(())
}
