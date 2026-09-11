use lariv_rs::db::trigram;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum JpxRssItems {
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
            "jpx_rss_items_title_trgm_idx",
            "jpx_rss_items",
            "title",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "jpx_rss_items_description_trgm_idx",
            "jpx_rss_items",
            "description",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "jpx_rss_items_guid_trgm_idx",
            "jpx_rss_items",
            "guid",
        )
        .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_jpx_rss_items_pub_date")
                    .table(JpxRssItems::Table)
                    .col(JpxRssItems::PubDate)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_jpx_rss_items_pub_date")
                    .table(JpxRssItems::Table)
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::drop_gin_index(db, backend, "jpx_rss_items_guid_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "jpx_rss_items_description_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "jpx_rss_items_title_trgm_idx").await
    }
}
