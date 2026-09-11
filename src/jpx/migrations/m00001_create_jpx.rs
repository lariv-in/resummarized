use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum JpxRssItems {
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
    ContentHash,
}

#[derive(DeriveIden)]
enum JpxFeedStatus {
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
                    .table(JpxRssItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(JpxRssItems::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(JpxRssItems::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(JpxRssItems::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(JpxRssItems::FeedKind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(JpxRssItems::Title).text().not_null())
                    .col(ColumnDef::new(JpxRssItems::Link).text().not_null())
                    .col(ColumnDef::new(JpxRssItems::Description).text().not_null())
                    .col(ColumnDef::new(JpxRssItems::PubDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(JpxRssItems::Guid).text())
                    .col(
                        ColumnDef::new(JpxRssItems::ContentHash)
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
                    .name("idx_jpx_rss_items_feed_kind")
                    .table(JpxRssItems::Table)
                    .col(JpxRssItems::FeedKind)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(JpxFeedStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(JpxFeedStatus::FeedKind)
                            .string_len(64)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(JpxFeedStatus::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(JpxFeedStatus::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(JpxFeedStatus::LastBuildDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(JpxFeedStatus::LastError).text())
                    .col(ColumnDef::new(JpxFeedStatus::LastFetchedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(JpxRssItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(JpxFeedStatus::Table).to_owned())
            .await
    }
}
