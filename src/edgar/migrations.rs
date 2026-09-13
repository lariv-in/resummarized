use sea_orm_migration::prelude::*;

use super::EdgarTag;

mod m00001_create_edgar;
mod m00002_edgar_pg_trgm;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_edgar::Migration),
            Box::new(m00002_edgar_pg_trgm::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: EdgarTag;
    migrator: Migrator;
}
