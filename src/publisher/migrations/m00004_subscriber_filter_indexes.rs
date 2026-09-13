use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Subscribers {
    Table,
    NewsletterInterval,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_subscribers_newsletter_interval")
                    .table(Subscribers::Table)
                    .col(Subscribers::NewsletterInterval)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_subscribers_newsletter_interval")
                    .table(Subscribers::Table)
                    .to_owned(),
            )
            .await
    }
}
