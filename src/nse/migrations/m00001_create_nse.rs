use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    FeedKind,
    Title,
    Link,
    Description,
    PubDate,
    ContentHash,
}

#[derive(DeriveIden)]
enum NseFeedStatus {
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
                    .table(NseRssItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NseRssItems::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NseRssItems::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(NseRssItems::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(NseRssItems::FeedKind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(NseRssItems::Title).text().not_null())
                    .col(ColumnDef::new(NseRssItems::Link).text().not_null())
                    .col(ColumnDef::new(NseRssItems::Description).text().not_null())
                    .col(ColumnDef::new(NseRssItems::PubDate).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(NseRssItems::ContentHash)
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
                    .name("idx_nse_rss_items_feed_kind")
                    .table(NseRssItems::Table)
                    .col(NseRssItems::FeedKind)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(NseFeedStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NseFeedStatus::FeedKind)
                            .string_len(64)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NseFeedStatus::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(NseFeedStatus::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(NseFeedStatus::LastBuildDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(NseFeedStatus::LastError).text())
                    .col(ColumnDef::new(NseFeedStatus::LastFetchedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NseRssItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(NseFeedStatus::Table).to_owned())
            .await
    }
}
