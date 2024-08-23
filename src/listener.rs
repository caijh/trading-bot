use application::application::{Application, RustApplication};
use application::application_context::ApplicationContext;
use application::application_event::{ApplicationEvenType, ApplicationEvent};
use application::application_listener::ApplicationListener;
use application::environment::Environment;
use database::DbService;
use database_common::connection::DbConnection;
use rbatis::rbdc::rt::block_on;
use std::error::Error;
use crate::job::jobs::load_jobs;

pub struct ApplicationContextInitializedListener {}

impl ApplicationListener for ApplicationContextInitializedListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::ContextInitialized
    }

    fn on_application_event(
        &self,
        application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        let application_context = application.get_application_context();
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
        event.get_event_type() == ApplicationEvenType::Started
    }

    fn on_application_event(
        &self,
        _application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {

        block_on(load_jobs())?;

        Ok(())
    }
}
