use application::application::RustApplication;
use application::application_context::{ApplicationContext, ConfigurableApplicationContext};
use application::application_event::{
    ApplicationContextInitializedEvent, ApplicationEvent, ApplicationStartedEvent,
};
use application::application_listener::ApplicationListener;
use application::environment::Environment;
use database::{DbConnection, DbService};
use std::any::TypeId;
use std::error::Error;

pub struct ApplicationContextInitializedListener {}

impl ApplicationListener for ApplicationContextInitializedListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_type_id() == TypeId::of::<ApplicationContextInitializedEvent>()
    }

    fn on_application_event(
        &self,
        _application: &RustApplication,
        application_context: &ConfigurableApplicationContext,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        let environment = application_context.get_environment();
        let db_connection = environment
            .get_property::<DbConnection>("database")
            .unwrap();
        let database_service = DbService::create_from_connection(db_connection);
        application_context.context.set(database_service);

        Ok(())
    }
}

pub struct ApplicationStartedEventListener {}

impl ApplicationListener for ApplicationStartedEventListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_type_id() == TypeId::of::<ApplicationStartedEvent>()
    }

    fn on_application_event(
        &self,
        _application: &RustApplication,
        _application_context: &ConfigurableApplicationContext,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        // block_on(load_jobs())?;

        Ok(())
    }
}
