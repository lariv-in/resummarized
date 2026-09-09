use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    UhpScripCode,
    UhpNseSymbol,
    UhpMseiSymbol,
    UhpSebiRegistration,
    UhpCompanyName,
    UhpTypeOfReport,
    UhpNumberOfSecurities,
    UhpReportingPeriodStart,
    UhpDateOfReport,
    UhpFyStart,
    UhpFyEnd,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::UhpScripCode,
            NseRssItems::UhpNseSymbol,
            NseRssItems::UhpMseiSymbol,
            NseRssItems::UhpSebiRegistration,
            NseRssItems::UhpCompanyName,
            NseRssItems::UhpTypeOfReport,
            NseRssItems::UhpNumberOfSecurities,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(NseRssItems::Table)
                        .add_column(ColumnDef::new(col).text())
                        .to_owned(),
                )
                .await?;
        }
        for col in [
            NseRssItems::UhpReportingPeriodStart,
            NseRssItems::UhpDateOfReport,
            NseRssItems::UhpFyStart,
            NseRssItems::UhpFyEnd,
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
            NseRssItems::UhpFyEnd,
            NseRssItems::UhpFyStart,
            NseRssItems::UhpDateOfReport,
            NseRssItems::UhpReportingPeriodStart,
            NseRssItems::UhpNumberOfSecurities,
            NseRssItems::UhpTypeOfReport,
            NseRssItems::UhpCompanyName,
            NseRssItems::UhpSebiRegistration,
            NseRssItems::UhpMseiSymbol,
            NseRssItems::UhpNseSymbol,
            NseRssItems::UhpScripCode,
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
