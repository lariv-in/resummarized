use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    SodNseSymbol,
    SodScripCode,
    SodMseiSymbol,
    SodIsin,
    SodCompanyName,
    SodStatementCount,
    SodQuarterEnded,
    SodModeOfFundRaising,
    SodDateOfFundsRaising,
    SodAmountRaised,
    SodMonitoringAgency,
    SodMonitoringAgencyName,
    SodHasDeviation,
    SodDeviationExplanation,
    SodShareholderApproved,
    SodAuditCommitteeComments,
    SodAuditorComments,
    SodObjects,
    SodSignatory,
    SodDesignation,
    SodPlace,
    SodDateOfSigning,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let text = [
            NseRssItems::SodNseSymbol,
            NseRssItems::SodScripCode,
            NseRssItems::SodMseiSymbol,
            NseRssItems::SodIsin,
            NseRssItems::SodCompanyName,
            NseRssItems::SodStatementCount,
            NseRssItems::SodModeOfFundRaising,
            NseRssItems::SodAmountRaised,
            NseRssItems::SodMonitoringAgency,
            NseRssItems::SodMonitoringAgencyName,
            NseRssItems::SodHasDeviation,
            NseRssItems::SodDeviationExplanation,
            NseRssItems::SodShareholderApproved,
            NseRssItems::SodAuditCommitteeComments,
            NseRssItems::SodAuditorComments,
            NseRssItems::SodObjects,
            NseRssItems::SodSignatory,
            NseRssItems::SodDesignation,
            NseRssItems::SodPlace,
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
        for col in [
            NseRssItems::SodQuarterEnded,
            NseRssItems::SodDateOfFundsRaising,
            NseRssItems::SodDateOfSigning,
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
            NseRssItems::SodDateOfSigning,
            NseRssItems::SodPlace,
            NseRssItems::SodDesignation,
            NseRssItems::SodSignatory,
            NseRssItems::SodObjects,
            NseRssItems::SodAuditorComments,
            NseRssItems::SodAuditCommitteeComments,
            NseRssItems::SodShareholderApproved,
            NseRssItems::SodDeviationExplanation,
            NseRssItems::SodHasDeviation,
            NseRssItems::SodMonitoringAgencyName,
            NseRssItems::SodMonitoringAgency,
            NseRssItems::SodAmountRaised,
            NseRssItems::SodDateOfFundsRaising,
            NseRssItems::SodModeOfFundRaising,
            NseRssItems::SodQuarterEnded,
            NseRssItems::SodStatementCount,
            NseRssItems::SodCompanyName,
            NseRssItems::SodIsin,
            NseRssItems::SodMseiSymbol,
            NseRssItems::SodScripCode,
            NseRssItems::SodNseSymbol,
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
