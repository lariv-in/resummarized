use sea_orm_migration::prelude::*;

use super::PublisherTag;

mod m00001_create_subscribers;
mod m00002_create_publisher_preferences;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_subscribers::Migration),
            Box::new(m00002_create_publisher_preferences::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: PublisherTag;
    migrator: Migrator;
}
