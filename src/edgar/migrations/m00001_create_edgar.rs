use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum EdgarItems {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    FeedKind,
    Title,
    Link,
    Description,
    PubDate,
    Guid,
    Category,
    Author,
    ContentHash,
}

#[derive(DeriveIden)]
enum EdgarFeedStatus {
    Table,
    FeedKind,
    CreatedAt,
    UpdatedAt,
    LastBuildDate,
    LastError,
    LastFetchedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EdgarItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EdgarItems::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EdgarItems::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(EdgarItems::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(EdgarItems::FeedKind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(EdgarItems::Title).text().not_null())
                    .col(ColumnDef::new(EdgarItems::Link).text().not_null())
                    .col(
                        ColumnDef::new(EdgarItems::Description)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EdgarItems::PubDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(EdgarItems::Guid).text())
                    .col(ColumnDef::new(EdgarItems::Category).text())
                    .col(ColumnDef::new(EdgarItems::Author).text())
                    .col(
                        ColumnDef::new(EdgarItems::ContentHash)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_edgar_items_feed_kind")
                    .table(EdgarItems::Table)
                    .col(EdgarItems::FeedKind)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EdgarFeedStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EdgarFeedStatus::FeedKind)
                            .string_len(64)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EdgarFeedStatus::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(EdgarFeedStatus::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(EdgarFeedStatus::LastBuildDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(EdgarFeedStatus::LastError).text())
                    .col(ColumnDef::new(EdgarFeedStatus::LastFetchedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EdgarItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(EdgarFeedStatus::Table).to_owned())
            .await
    }
}
