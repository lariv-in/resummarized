use lariv_rs::db::trigram;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NseRssItems {
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
            "nse_rss_items_title_trgm_idx",
            "nse_rss_items",
            "title",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "nse_rss_items_description_trgm_idx",
            "nse_rss_items",
            "description",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "nse_rss_items_subject_trgm_idx",
            "nse_rss_items",
            "subject",
        )
        .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_nse_rss_items_pub_date")
                    .table(NseRssItems::Table)
                    .col(NseRssItems::PubDate)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_nse_rss_items_pub_date")
                    .table(NseRssItems::Table)
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::drop_gin_index(db, backend, "nse_rss_items_subject_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "nse_rss_items_description_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "nse_rss_items_title_trgm_idx").await
    }
}
