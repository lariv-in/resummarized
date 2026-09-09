use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum BseRssItems {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    FeedKind,
    Title,
    Link,
    Description,
    PubDate,
    Scripcode,
    AsOnDate,
    MeetingDate,
    MeetingType,
    Purpose,
    Segment,
    RdDate,
    BcStartDate,
    BcEndDate,
    NdStartDate,
    NdEndDate,
    ActualPaymentDate,
    TypeOfSecurity,
    AuditedUnaudited,
    StandaloneConsolidated,
    PeriodStartDate,
    PeriodEndDate,
    IndAs,
    PromoterAndGroup,
    PublicVal,
    Emptr,
    Status,
    SubmissionDate,
    RevisedFilingDate,
    ContentHash,
}

#[derive(DeriveIden)]
enum BseFeedStatus {
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
                    .table(BseRssItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BseRssItems::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BseRssItems::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(BseRssItems::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(BseRssItems::FeedKind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(BseRssItems::Title).text().not_null())
                    .col(ColumnDef::new(BseRssItems::Link).text().not_null())
                    .col(ColumnDef::new(BseRssItems::Description).text().not_null())
                    .col(ColumnDef::new(BseRssItems::PubDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(BseRssItems::Scripcode).text())
                    .col(ColumnDef::new(BseRssItems::AsOnDate).date())
                    .col(ColumnDef::new(BseRssItems::MeetingDate).date())
                    .col(ColumnDef::new(BseRssItems::MeetingType).text())
                    .col(ColumnDef::new(BseRssItems::Purpose).text())
                    .col(ColumnDef::new(BseRssItems::Segment).text())
                    .col(ColumnDef::new(BseRssItems::RdDate).date())
                    .col(ColumnDef::new(BseRssItems::BcStartDate).date())
                    .col(ColumnDef::new(BseRssItems::BcEndDate).date())
                    .col(ColumnDef::new(BseRssItems::NdStartDate).date())
                    .col(ColumnDef::new(BseRssItems::NdEndDate).date())
                    .col(ColumnDef::new(BseRssItems::ActualPaymentDate).date())
                    .col(ColumnDef::new(BseRssItems::TypeOfSecurity).text())
                    .col(ColumnDef::new(BseRssItems::AuditedUnaudited).text())
                    .col(ColumnDef::new(BseRssItems::StandaloneConsolidated).text())
                    .col(ColumnDef::new(BseRssItems::PeriodStartDate).date())
                    .col(ColumnDef::new(BseRssItems::PeriodEndDate).date())
                    .col(ColumnDef::new(BseRssItems::IndAs).text())
                    .col(ColumnDef::new(BseRssItems::PromoterAndGroup).text())
                    .col(ColumnDef::new(BseRssItems::PublicVal).text())
                    .col(ColumnDef::new(BseRssItems::Emptr).text())
                    .col(ColumnDef::new(BseRssItems::Status).text())
                    .col(ColumnDef::new(BseRssItems::SubmissionDate).date())
                    .col(ColumnDef::new(BseRssItems::RevisedFilingDate).date())
                    .col(
                        ColumnDef::new(BseRssItems::ContentHash)
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
                    .name("idx_bse_rss_items_feed_kind")
                    .table(BseRssItems::Table)
                    .col(BseRssItems::FeedKind)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BseFeedStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BseFeedStatus::FeedKind)
                            .string_len(64)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BseFeedStatus::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(BseFeedStatus::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(BseFeedStatus::LastBuildDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(BseFeedStatus::LastError).text())
                    .col(ColumnDef::new(BseFeedStatus::LastFetchedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BseRssItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BseFeedStatus::Table).to_owned())
            .await
    }
}
