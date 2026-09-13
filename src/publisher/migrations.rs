use sea_orm_migration::prelude::*;

use super::PublisherTag;

mod m00001_create_subscribers;
mod m00002_create_publisher_preferences;
mod m00003_subscriber_filters;
mod m00004_subscriber_filter_indexes;
mod m00005_subscriber_one_time_token;
mod m00006_subscription_email_templates;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_subscribers::Migration),
            Box::new(m00002_create_publisher_preferences::Migration),
            Box::new(m00003_subscriber_filters::Migration),
            Box::new(m00004_subscriber_filter_indexes::Migration),
            Box::new(m00005_subscriber_one_time_token::Migration),
            Box::new(m00006_subscription_email_templates::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: PublisherTag;
    migrator: Migrator;
}
