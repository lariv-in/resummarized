use lariv_rs::db::trigram;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum EdgarItems {
    Table,
    PubDate,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::create_gin_index(
            db,
            backend,
            "edgar_items_title_trgm_idx",
            "edgar_items",
            "title",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "edgar_items_description_trgm_idx",
            "edgar_items",
            "description",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "edgar_items_category_trgm_idx",
            "edgar_items",
            "category",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "edgar_items_guid_trgm_idx",
            "edgar_items",
            "guid",
        )
        .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_edgar_items_pub_date")
                    .table(EdgarItems::Table)
                    .col(EdgarItems::PubDate)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_edgar_items_pub_date")
                    .table(EdgarItems::Table)
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::drop_gin_index(db, backend, "edgar_items_guid_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "edgar_items_category_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "edgar_items_description_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "edgar_items_title_trgm_idx").await
    }
}
