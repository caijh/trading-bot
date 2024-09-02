use crate::job::jobs::load_jobs;
use application::application::{Application, RustApplication};
use application::context::application_context::ApplicationContext;
use application::context::application_event::{ApplicationEvenType, ApplicationEvent};
use application::context::application_listener::ApplicationListener;
use application::env::property_resolver::PropertyResolver;
use async_trait::async_trait;
use database::DbService;
use database_common::connection::DbConnection;
use std::error::Error;

pub struct ApplicationContextInitializedListener {}

#[async_trait]
impl ApplicationListener for ApplicationContextInitializedListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::ContextInitialized
    }

    async fn on_application_event(
        &self,
        application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        let application_context = application.get_application_context();
        let environment = application_context.get_environment().await;
        let db_connection = environment
            .get_property::<DbConnection>("database")
            .unwrap();
        let database_service = DbService::create_from_connection(db_connection);
        application_context.context.set(database_service);

        Ok(())
    }
}

pub struct ApplicationStartedEventListener {}

#[async_trait]
impl ApplicationListener for ApplicationStartedEventListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::Started
    }

    async fn on_application_event(
        &self,
        _application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn Error>> {
        load_jobs().await?;

        Ok(())
    }
}
