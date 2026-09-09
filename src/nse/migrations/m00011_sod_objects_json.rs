use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    SodModifiedObject,
    SodOriginalAllocation,
    SodModifiedAllocation,
    SodFundsUtilised,
    SodAmountOfDeviation,
    SodObjectNotes,
}

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute(Statement::from_string(
            manager.get_database_backend(),
            sql.to_string(),
        ))
        .await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::SodObjectNotes,
            NseRssItems::SodAmountOfDeviation,
            NseRssItems::SodFundsUtilised,
            NseRssItems::SodModifiedAllocation,
            NseRssItems::SodOriginalAllocation,
            NseRssItems::SodModifiedObject,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }
        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                exec(
                    manager,
                    "ALTER TABLE nse_rss_items ALTER COLUMN sod_objects TYPE jsonb USING NULL::jsonb",
                )
                .await?;
            }
            DatabaseBackend::Sqlite => {
                exec(manager, "UPDATE nse_rss_items SET sod_objects = NULL").await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                exec(
                    manager,
                    "ALTER TABLE nse_rss_items ALTER COLUMN sod_objects TYPE text USING sod_objects::text",
                )
                .await?;
            }
            DatabaseBackend::Sqlite => {
                exec(manager, "UPDATE nse_rss_items SET sod_objects = NULL").await?;
            }
            _ => {}
        }
        for col in [
            NseRssItems::SodModifiedObject,
            NseRssItems::SodOriginalAllocation,
            NseRssItems::SodModifiedAllocation,
            NseRssItems::SodFundsUtilised,
            NseRssItems::SodAmountOfDeviation,
            NseRssItems::SodObjectNotes,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .add_column(ColumnDef::new(col).text())
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
