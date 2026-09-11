use sea_orm_migration::prelude::*;

use super::EuronextTag;

mod m00001_create_euronext;
mod m00002_euronext_pg_trgm;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_euronext::Migration),
            Box::new(m00002_euronext_pg_trgm::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: EuronextTag;
    migrator: Migrator;
}
