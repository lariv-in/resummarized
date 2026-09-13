use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Subscribers {
    Table,
    FilterExchanges,
    FilterEntities,
    FilterEventTypes,
    NewsletterInterval,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .add_column(ColumnDef::new(Subscribers::FilterExchanges).json())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .add_column(ColumnDef::new(Subscribers::FilterEntities).json())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .add_column(ColumnDef::new(Subscribers::FilterEventTypes).json())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .add_column(
                        ColumnDef::new(Subscribers::NewsletterInterval)
                            .text()
                            .not_null()
                            .default("weekly"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .drop_column(Subscribers::NewsletterInterval)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .drop_column(Subscribers::FilterEventTypes)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .drop_column(Subscribers::FilterEntities)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .drop_column(Subscribers::FilterExchanges)
                    .to_owned(),
            )
            .await
    }
}
