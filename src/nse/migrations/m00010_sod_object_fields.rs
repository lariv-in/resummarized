use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
    Table,
    SodModifiedObject,
    SodOriginalAllocation,
    SodModifiedAllocation,
    SodFundsUtilised,
    SodAmountOfDeviation,
    SodObjectNotes,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::SodModifiedObject,
            NseRssItems::SodOriginalAllocation,
            NseRssItems::SodModifiedAllocation,
            NseRssItems::SodFundsUtilised,
            NseRssItems::SodAmountOfDeviation,
            NseRssItems::SodObjectNotes,
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
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            NseRssItems::SodObjectNotes,
            NseRssItems::SodAmountOfDeviation,
            NseRssItems::SodFundsUtilised,
            NseRssItems::SodModifiedAllocation,
            NseRssItems::SodOriginalAllocation,
            NseRssItems::SodModifiedObject,
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
