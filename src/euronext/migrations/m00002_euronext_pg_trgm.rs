use lariv_rs::db::trigram;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum EuronextRssItems {
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
            "euronext_rss_items_title_trgm_idx",
            "euronext_rss_items",
            "title",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "euronext_rss_items_description_trgm_idx",
            "euronext_rss_items",
            "description",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "euronext_rss_items_guid_trgm_idx",
            "euronext_rss_items",
            "guid",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "euronext_rss_items_company_name_trgm_idx",
            "euronext_rss_items",
            "company_name",
        )
        .await?;
        trigram::create_gin_index(
            db,
            backend,
            "euronext_rss_items_company_ticker_trgm_idx",
            "euronext_rss_items",
            "company_ticker",
        )
        .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_euronext_rss_items_pub_date")
                    .table(EuronextRssItems::Table)
                    .col(EuronextRssItems::PubDate)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_euronext_rss_items_pub_date")
                    .table(EuronextRssItems::Table)
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::drop_gin_index(db, backend, "euronext_rss_items_company_ticker_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "euronext_rss_items_company_name_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "euronext_rss_items_guid_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "euronext_rss_items_description_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "euronext_rss_items_title_trgm_idx").await
    }
}
