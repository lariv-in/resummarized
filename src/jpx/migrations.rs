use sea_orm_migration::prelude::*;

use super::JpxTag;

mod m00001_create_jpx;
mod m00002_jpx_pg_trgm;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_jpx::Migration),
            Box::new(m00002_jpx_pg_trgm::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: JpxTag;
    migrator: Migrator;
}
