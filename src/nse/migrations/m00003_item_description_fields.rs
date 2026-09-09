use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    Subject,
    OriginalSubmissionDate,
    Series,
    Purpose,
    FaceValue,
    RecordDate,
    BookClosureStartDate,
    BookClosureEndDate,
    RelatingTo,
    AuditedUnaudited,
    Cumulative,
    Consolidated,
    IndAs,
    Period,
    PeriodEnded,
    ForQuarterEnding,
    EncumberedPromoterNames,
    PeriodEndDate,
    AcquirerNames,
    PromoterNames,
    FinancialYear,
    SubmissionType,
    MeetingDate,
    Remarks,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let text = [
            NseRssItems::Subject,
            NseRssItems::Series,
            NseRssItems::Purpose,
            NseRssItems::FaceValue,
            NseRssItems::RelatingTo,
            NseRssItems::AuditedUnaudited,
            NseRssItems::Cumulative,
            NseRssItems::Consolidated,
            NseRssItems::IndAs,
            NseRssItems::Period,
            NseRssItems::EncumberedPromoterNames,
            NseRssItems::AcquirerNames,
            NseRssItems::PromoterNames,
            NseRssItems::FinancialYear,
            NseRssItems::SubmissionType,
            NseRssItems::Remarks,
        ];
        for col in text {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .add_column(ColumnDef::new(col).text())
                        .to_owned(),
                )
                .await?;
        }
        manager
            .alter_table(
                Table::alter()
                    .table(NseRssItems::Table)
                    .add_column(
                        ColumnDef::new(NseRssItems::OriginalSubmissionDate)
                            .timestamp_with_time_zone(),
                    )
                    .to_owned(),
            )
            .await?;
        for col in [
            NseRssItems::RecordDate,
            NseRssItems::BookClosureStartDate,
            NseRssItems::BookClosureEndDate,
            NseRssItems::PeriodEnded,
            NseRssItems::ForQuarterEnding,
            NseRssItems::PeriodEndDate,
            NseRssItems::MeetingDate,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .add_column(ColumnDef::new(col).date())
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::Remarks,
            NseRssItems::MeetingDate,
            NseRssItems::SubmissionType,
            NseRssItems::FinancialYear,
            NseRssItems::PromoterNames,
            NseRssItems::AcquirerNames,
            NseRssItems::PeriodEndDate,
            NseRssItems::EncumberedPromoterNames,
            NseRssItems::ForQuarterEnding,
            NseRssItems::PeriodEnded,
            NseRssItems::Period,
            NseRssItems::IndAs,
            NseRssItems::Consolidated,
            NseRssItems::Cumulative,
            NseRssItems::AuditedUnaudited,
            NseRssItems::RelatingTo,
            NseRssItems::BookClosureEndDate,
            NseRssItems::BookClosureStartDate,
            NseRssItems::RecordDate,
            NseRssItems::FaceValue,
            NseRssItems::Purpose,
            NseRssItems::Series,
            NseRssItems::OriginalSubmissionDate,
            NseRssItems::Subject,
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
        Ok(())
    }
}
