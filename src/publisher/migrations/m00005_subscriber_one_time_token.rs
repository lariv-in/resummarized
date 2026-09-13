use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Subscribers {
    Table,
    OneTimeToken,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .add_column(ColumnDef::new(Subscribers::OneTimeToken).uuid())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_subscribers_one_time_token")
                    .table(Subscribers::Table)
                    .col(Subscribers::OneTimeToken)
                    .to_owned(),
            )
            .await?;

        // Backfill existing rows if running against PostgreSQL
        let db_backend = manager.get_database_backend();
        if db_backend == sea_orm::DatabaseBackend::Postgres {
            let _ = manager
                .get_connection()
                .execute(Statement::from_string(
                    db_backend,
                    "UPDATE subscribers SET one_time_token = gen_random_uuid() WHERE one_time_token IS NULL",
                ))
                .await;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_subscribers_one_time_token")
                    .table(Subscribers::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .drop_column(Subscribers::OneTimeToken)
                    .to_owned(),
            )
            .await
    }
}
