use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    ScrNseSymbol,
    ScrScripCode,
    ScrMseiSymbol,
    ScrIsin,
    ScrCompanyName,
    ScrFyStart,
    ScrFyEnd,
    ScrDateOfReport,
    ScrObservationsReported,
    ScrPreviousObservations,
    ScrActionsTaken,
    ScrCertifyingFirm,
    ScrPcsName,
    ScrMembershipType,
    ScrMembershipNumber,
    ScrUdin,
    ScrCpNumber,
    ScrPlace,
    ScrPcsReportDate,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::ScrNseSymbol,
            NseRssItems::ScrScripCode,
            NseRssItems::ScrMseiSymbol,
            NseRssItems::ScrIsin,
            NseRssItems::ScrCompanyName,
            NseRssItems::ScrObservationsReported,
            NseRssItems::ScrPreviousObservations,
            NseRssItems::ScrActionsTaken,
            NseRssItems::ScrCertifyingFirm,
            NseRssItems::ScrPcsName,
            NseRssItems::ScrMembershipType,
            NseRssItems::ScrMembershipNumber,
            NseRssItems::ScrUdin,
            NseRssItems::ScrCpNumber,
            NseRssItems::ScrPlace,
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
            NseRssItems::ScrFyStart,
            NseRssItems::ScrFyEnd,
            NseRssItems::ScrDateOfReport,
            NseRssItems::ScrPcsReportDate,
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
            NseRssItems::ScrPcsReportDate,
            NseRssItems::ScrDateOfReport,
            NseRssItems::ScrFyEnd,
            NseRssItems::ScrFyStart,
            NseRssItems::ScrPlace,
            NseRssItems::ScrCpNumber,
            NseRssItems::ScrUdin,
            NseRssItems::ScrMembershipNumber,
            NseRssItems::ScrMembershipType,
            NseRssItems::ScrPcsName,
            NseRssItems::ScrCertifyingFirm,
            NseRssItems::ScrActionsTaken,
            NseRssItems::ScrPreviousObservations,
            NseRssItems::ScrObservationsReported,
            NseRssItems::ScrCompanyName,
            NseRssItems::ScrIsin,
            NseRssItems::ScrMseiSymbol,
            NseRssItems::ScrScripCode,
            NseRssItems::ScrNseSymbol,
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
