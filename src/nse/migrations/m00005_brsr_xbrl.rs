use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    BrsrNseSymbol,
    BrsrScripCode,
    BrsrMseiSymbol,
    BrsrIsin,
    BrsrCin,
    BrsrCompanyName,
    BrsrDateOfIncorporation,
    BrsrRegisteredOffice,
    BrsrCorporateOffice,
    BrsrEmail,
    BrsrTelephone,
    BrsrWebsite,
    BrsrFyStart,
    BrsrFyEnd,
    BrsrPyStart,
    BrsrPyEnd,
    BrsrPpyStart,
    BrsrPpyEnd,
    BrsrPaidUpCapital,
    BrsrContactPerson,
    BrsrContactPhone,
    BrsrContactEmail,
    BrsrReportingBoundary,
    BrsrCoreAssurance,
    BrsrTurnover,
    BrsrNetWorth,
    BrsrStatesServed,
    BrsrCountriesServed,
    BrsrBoardSize,
    BrsrFemaleDirectors,
    BrsrKmp,
    BrsrFemaleKmp,
    BrsrCsrApplicable,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let text = [
            NseRssItems::BrsrNseSymbol,
            NseRssItems::BrsrScripCode,
            NseRssItems::BrsrMseiSymbol,
            NseRssItems::BrsrIsin,
            NseRssItems::BrsrCin,
            NseRssItems::BrsrCompanyName,
            NseRssItems::BrsrRegisteredOffice,
            NseRssItems::BrsrCorporateOffice,
            NseRssItems::BrsrEmail,
            NseRssItems::BrsrTelephone,
            NseRssItems::BrsrWebsite,
            NseRssItems::BrsrPaidUpCapital,
            NseRssItems::BrsrContactPerson,
            NseRssItems::BrsrContactPhone,
            NseRssItems::BrsrContactEmail,
            NseRssItems::BrsrReportingBoundary,
            NseRssItems::BrsrCoreAssurance,
            NseRssItems::BrsrTurnover,
            NseRssItems::BrsrNetWorth,
            NseRssItems::BrsrStatesServed,
            NseRssItems::BrsrCountriesServed,
            NseRssItems::BrsrBoardSize,
            NseRssItems::BrsrFemaleDirectors,
            NseRssItems::BrsrKmp,
            NseRssItems::BrsrFemaleKmp,
            NseRssItems::BrsrCsrApplicable,
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
            NseRssItems::BrsrDateOfIncorporation,
            NseRssItems::BrsrFyStart,
            NseRssItems::BrsrFyEnd,
            NseRssItems::BrsrPyStart,
            NseRssItems::BrsrPyEnd,
            NseRssItems::BrsrPpyStart,
            NseRssItems::BrsrPpyEnd,
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
            NseRssItems::BrsrCsrApplicable,
            NseRssItems::BrsrFemaleKmp,
            NseRssItems::BrsrKmp,
            NseRssItems::BrsrFemaleDirectors,
            NseRssItems::BrsrBoardSize,
            NseRssItems::BrsrCountriesServed,
            NseRssItems::BrsrStatesServed,
            NseRssItems::BrsrNetWorth,
            NseRssItems::BrsrTurnover,
            NseRssItems::BrsrCoreAssurance,
            NseRssItems::BrsrReportingBoundary,
            NseRssItems::BrsrContactEmail,
            NseRssItems::BrsrContactPhone,
            NseRssItems::BrsrContactPerson,
            NseRssItems::BrsrPaidUpCapital,
            NseRssItems::BrsrPpyEnd,
            NseRssItems::BrsrPpyStart,
            NseRssItems::BrsrPyEnd,
            NseRssItems::BrsrPyStart,
            NseRssItems::BrsrFyEnd,
            NseRssItems::BrsrFyStart,
            NseRssItems::BrsrWebsite,
            NseRssItems::BrsrTelephone,
            NseRssItems::BrsrEmail,
            NseRssItems::BrsrCorporateOffice,
            NseRssItems::BrsrRegisteredOffice,
            NseRssItems::BrsrDateOfIncorporation,
            NseRssItems::BrsrCompanyName,
            NseRssItems::BrsrCin,
            NseRssItems::BrsrIsin,
            NseRssItems::BrsrMseiSymbol,
            NseRssItems::BrsrScripCode,
            NseRssItems::BrsrNseSymbol,
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
