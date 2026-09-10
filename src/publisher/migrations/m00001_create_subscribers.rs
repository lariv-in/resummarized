use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Subscribers {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Email,
    SubscriptionDate,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscribers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Subscribers::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Subscribers::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Subscribers::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Subscribers::Email).text().not_null())
                    .col(
                        ColumnDef::new(Subscribers::SubscriptionDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("uix_subscribers_email")
                    .table(Subscribers::Table)
                    .col(Subscribers::Email)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscribers::Table).to_owned())
            .await
    }
}
