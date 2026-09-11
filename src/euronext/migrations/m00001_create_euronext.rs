use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum EuronextRssItems {
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
    CompanyName,
    CompanyTicker,
    Attachment,
    AttachmentTitle,
    ContentHash,
}

#[derive(DeriveIden)]
enum EuronextFeedStatus {
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
                    .table(EuronextRssItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EuronextRssItems::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EuronextRssItems::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(EuronextRssItems::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(EuronextRssItems::FeedKind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(EuronextRssItems::Title).text().not_null())
                    .col(ColumnDef::new(EuronextRssItems::Link).text().not_null())
                    .col(
                        ColumnDef::new(EuronextRssItems::Description)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EuronextRssItems::PubDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(EuronextRssItems::Guid).text())
                    .col(ColumnDef::new(EuronextRssItems::CompanyName).text())
                    .col(ColumnDef::new(EuronextRssItems::CompanyTicker).text())
                    .col(ColumnDef::new(EuronextRssItems::Attachment).text())
                    .col(ColumnDef::new(EuronextRssItems::AttachmentTitle).text())
                    .col(
                        ColumnDef::new(EuronextRssItems::ContentHash)
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
                    .name("idx_euronext_rss_items_feed_kind")
                    .table(EuronextRssItems::Table)
                    .col(EuronextRssItems::FeedKind)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EuronextFeedStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EuronextFeedStatus::FeedKind)
                            .string_len(64)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EuronextFeedStatus::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(EuronextFeedStatus::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(EuronextFeedStatus::LastBuildDate)
                            .timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(EuronextFeedStatus::LastError).text())
                    .col(
                        ColumnDef::new(EuronextFeedStatus::LastFetchedAt)
                            .timestamp_with_time_zone(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EuronextRssItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(EuronextFeedStatus::Table).to_owned())
            .await
    }
}
