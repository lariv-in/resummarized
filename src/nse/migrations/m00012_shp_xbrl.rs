use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    ShpNseSymbol,
    ShpScripCode,
    ShpMseiSymbol,
    ShpIsin,
    ShpCompanyName,
    ShpClassOfSecurity,
    ShpTypeOfReport,
    ShpDateOfReport,
    ShpFiledUnder,
    ShpPromoterPct,
    ShpPublicPct,
    ShpPromoterShares,
    ShpPublicShares,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::ShpNseSymbol,
            NseRssItems::ShpScripCode,
            NseRssItems::ShpMseiSymbol,
            NseRssItems::ShpIsin,
            NseRssItems::ShpCompanyName,
            NseRssItems::ShpClassOfSecurity,
            NseRssItems::ShpTypeOfReport,
            NseRssItems::ShpFiledUnder,
            NseRssItems::ShpPromoterPct,
            NseRssItems::ShpPublicPct,
            NseRssItems::ShpPromoterShares,
            NseRssItems::ShpPublicShares,
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
        manager
            .alter_table(
                Table::alter()
                    .table(NseRssItems::Table)
                    .add_column(ColumnDef::new(NseRssItems::ShpDateOfReport).date())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::ShpDateOfReport,
            NseRssItems::ShpPublicShares,
            NseRssItems::ShpPromoterShares,
            NseRssItems::ShpPublicPct,
            NseRssItems::ShpPromoterPct,
            NseRssItems::ShpFiledUnder,
            NseRssItems::ShpTypeOfReport,
            NseRssItems::ShpClassOfSecurity,
            NseRssItems::ShpCompanyName,
            NseRssItems::ShpIsin,
            NseRssItems::ShpMseiSymbol,
            NseRssItems::ShpScripCode,
            NseRssItems::ShpNseSymbol,
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
