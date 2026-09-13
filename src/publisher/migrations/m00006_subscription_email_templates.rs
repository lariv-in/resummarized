use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PublisherPreferences {
    Table,
    EditLinkEmailSubject,
    EditLinkEmailTemplate,
    EditOpenedEmailSubject,
    EditOpenedEmailTemplate,
    EditUpdatedEmailSubject,
    EditUpdatedEmailTemplate,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let columns = [
            (PublisherPreferences::EditLinkEmailSubject, ""),
            (PublisherPreferences::EditLinkEmailTemplate, ""),
            (PublisherPreferences::EditOpenedEmailSubject, ""),
            (PublisherPreferences::EditOpenedEmailTemplate, ""),
            (PublisherPreferences::EditUpdatedEmailSubject, ""),
            (PublisherPreferences::EditUpdatedEmailTemplate, ""),
        ];

        for (col, def) in columns {
            manager
                .alter_table(
                    Table::alter()
                        .table(PublisherPreferences::Table)
                        .add_column(
                            ColumnDef::new(col)
                                .text()
                                .not_null()
                                .default(def),
                        )
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let columns = [
            PublisherPreferences::EditLinkEmailSubject,
            PublisherPreferences::EditLinkEmailTemplate,
            PublisherPreferences::EditOpenedEmailSubject,
            PublisherPreferences::EditOpenedEmailTemplate,
            PublisherPreferences::EditUpdatedEmailSubject,
            PublisherPreferences::EditUpdatedEmailTemplate,
        ];

        for col in columns {
            manager
                .alter_table(
                    Table::alter()
                        .table(PublisherPreferences::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
