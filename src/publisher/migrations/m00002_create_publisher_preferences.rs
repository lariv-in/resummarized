use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PublisherPreferences {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    HtmlTemplate,
    SmtpHost,
    SmtpPort,
    SmtpUsername,
    SmtpPassword,
    SmtpFrom,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PublisherPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PublisherPreferences::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PublisherPreferences::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(PublisherPreferences::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(PublisherPreferences::HtmlTemplate)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(PublisherPreferences::SmtpHost)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(PublisherPreferences::SmtpPort)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(PublisherPreferences::SmtpUsername)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(PublisherPreferences::SmtpPassword)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(PublisherPreferences::SmtpFrom)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PublisherPreferences::Table).to_owned())
            .await
    }
}
