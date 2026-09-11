use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NasdaqRssItems {
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
enum NasdaqFeedStatus {
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
                    .table(NasdaqRssItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NasdaqRssItems::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NasdaqRssItems::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(NasdaqRssItems::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(NasdaqRssItems::FeedKind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(NasdaqRssItems::Title).text().not_null())
                    .col(ColumnDef::new(NasdaqRssItems::Link).text().not_null())
                    .col(
                        ColumnDef::new(NasdaqRssItems::Description)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NasdaqRssItems::PubDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(NasdaqRssItems::Guid).text())
                    .col(ColumnDef::new(NasdaqRssItems::Category).text())
                    .col(ColumnDef::new(NasdaqRssItems::Author).text())
                    .col(
                        ColumnDef::new(NasdaqRssItems::ContentHash)
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
                    .name("idx_nasdaq_rss_items_feed_kind")
                    .table(NasdaqRssItems::Table)
                    .col(NasdaqRssItems::FeedKind)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(NasdaqFeedStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NasdaqFeedStatus::FeedKind)
                            .string_len(64)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NasdaqFeedStatus::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(NasdaqFeedStatus::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(NasdaqFeedStatus::LastBuildDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(NasdaqFeedStatus::LastError).text())
                    .col(ColumnDef::new(NasdaqFeedStatus::LastFetchedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NasdaqRssItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(NasdaqFeedStatus::Table).to_owned())
            .await
    }
}
