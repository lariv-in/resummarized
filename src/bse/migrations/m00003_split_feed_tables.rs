use lariv_rs::db::trigram;
use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn exec(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute(Statement::from_string(
            manager.get_database_backend(),
            sql.to_string(),
        ))
        .await?;
    Ok(())
}

async fn create_sat(
    manager: &SchemaManager<'_>,
    table: &str,
    cols: impl FnOnce(&mut TableCreateStatement),
) -> Result<(), DbErr> {
    let mut t = Table::create();
    t.table(Alias::new(table))
        .if_not_exists()
        .col(
            ColumnDef::new(Alias::new("item_id"))
                .big_integer()
                .not_null()
                .primary_key(),
        )
        .foreign_key(
            ForeignKey::create()
                .name(format!("fk_{table}_item"))
                .from(Alias::new(table), Alias::new("item_id"))
                .to(Alias::new("bse_rss_items"), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade),
        );
    cols(&mut t);
    manager.create_table(t.to_owned()).await
}

async fn drop_parent_col(manager: &SchemaManager<'_>, col: &str) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(Alias::new("bse_rss_items"))
                .drop_column(Alias::new(col))
                .to_owned(),
        )
        .await
}

async fn add_parent_text(manager: &SchemaManager<'_>, col: &str) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(Alias::new("bse_rss_items"))
                .add_column(ColumnDef::new(Alias::new(col)).text())
                .to_owned(),
        )
        .await
}

async fn add_parent_date(manager: &SchemaManager<'_>, col: &str) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(Alias::new("bse_rss_items"))
                .add_column(ColumnDef::new(Alias::new(col)).date())
                .to_owned(),
        )
        .await
}

const TEXT_COLS: &[&str] = &[
    "scripcode",
    "meeting_type",
    "purpose",
    "segment",
    "type_of_security",
    "audited_unaudited",
    "standalone_consolidated",
    "ind_as",
    "promoter_and_group",
    "public_val",
    "emptr",
    "status",
];

const DATE_COLS: &[&str] = &[
    "as_on_date",
    "meeting_date",
    "rd_date",
    "bc_start_date",
    "bc_end_date",
    "nd_start_date",
    "nd_end_date",
    "actual_payment_date",
    "period_start_date",
    "period_end_date",
    "submission_date",
    "revised_filing_date",
];

const SAT_TRGM: &[(&str, &str, &str)] = &[
    (
        "bse_announcements_scripcode_trgm_idx",
        "bse_announcements",
        "scripcode",
    ),
    (
        "bse_annual_reports_scripcode_trgm_idx",
        "bse_annual_reports",
        "scripcode",
    ),
    (
        "bse_board_meetings_scripcode_trgm_idx",
        "bse_board_meetings",
        "scripcode",
    ),
    (
        "bse_board_meetings_purpose_trgm_idx",
        "bse_board_meetings",
        "purpose",
    ),
    (
        "bse_corporate_actions_scripcode_trgm_idx",
        "bse_corporate_actions",
        "scripcode",
    ),
    (
        "bse_corporate_actions_segment_trgm_idx",
        "bse_corporate_actions",
        "segment",
    ),
    (
        "bse_corporate_actions_purpose_trgm_idx",
        "bse_corporate_actions",
        "purpose",
    ),
    (
        "bse_financial_results_scripcode_trgm_idx",
        "bse_financial_results",
        "scripcode",
    ),
    (
        "bse_financial_results_audited_unaudited_trgm_idx",
        "bse_financial_results",
        "audited_unaudited",
    ),
    (
        "bse_financial_results_standalone_consolidated_trgm_idx",
        "bse_financial_results",
        "standalone_consolidated",
    ),
    (
        "bse_financial_results_ind_as_trgm_idx",
        "bse_financial_results",
        "ind_as",
    ),
    (
        "bse_insider_trading_scripcode_trgm_idx",
        "bse_insider_trading",
        "scripcode",
    ),
    (
        "bse_insider_trading_type_of_security_trgm_idx",
        "bse_insider_trading",
        "type_of_security",
    ),
    (
        "bse_shareholding_pattern_scripcode_trgm_idx",
        "bse_shareholding_pattern",
        "scripcode",
    ),
    (
        "bse_shareholding_pattern_promoter_and_group_trgm_idx",
        "bse_shareholding_pattern",
        "promoter_and_group",
    ),
    (
        "bse_shareholding_pattern_public_val_trgm_idx",
        "bse_shareholding_pattern",
        "public_val",
    ),
    (
        "bse_shareholding_pattern_emptr_trgm_idx",
        "bse_shareholding_pattern",
        "emptr",
    ),
    (
        "bse_shareholding_pattern_status_trgm_idx",
        "bse_shareholding_pattern",
        "status",
    ),
    (
        "bse_voting_results_scripcode_trgm_idx",
        "bse_voting_results",
        "scripcode",
    ),
    (
        "bse_voting_results_meeting_type_trgm_idx",
        "bse_voting_results",
        "meeting_type",
    ),
];

const SAT_TABLES: &[&str] = &[
    "bse_announcements",
    "bse_annual_reports",
    "bse_board_meetings",
    "bse_corporate_actions",
    "bse_financial_results",
    "bse_insider_trading",
    "bse_shareholding_pattern",
    "bse_voting_results",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_sat(manager, "bse_announcements", |t| {
            t.col(ColumnDef::new(Alias::new("scripcode")).text());
        })
        .await?;
        create_sat(manager, "bse_annual_reports", |t| {
            t.col(ColumnDef::new(Alias::new("scripcode")).text());
            t.col(ColumnDef::new(Alias::new("as_on_date")).date());
        })
        .await?;
        create_sat(manager, "bse_board_meetings", |t| {
            t.col(ColumnDef::new(Alias::new("scripcode")).text());
            t.col(ColumnDef::new(Alias::new("meeting_date")).date());
            t.col(ColumnDef::new(Alias::new("purpose")).text());
        })
        .await?;
        create_sat(manager, "bse_corporate_actions", |t| {
            t.col(ColumnDef::new(Alias::new("scripcode")).text());
            t.col(ColumnDef::new(Alias::new("segment")).text());
            t.col(ColumnDef::new(Alias::new("purpose")).text());
            t.col(ColumnDef::new(Alias::new("rd_date")).date());
            t.col(ColumnDef::new(Alias::new("bc_start_date")).date());
            t.col(ColumnDef::new(Alias::new("bc_end_date")).date());
            t.col(ColumnDef::new(Alias::new("nd_start_date")).date());
            t.col(ColumnDef::new(Alias::new("nd_end_date")).date());
            t.col(ColumnDef::new(Alias::new("actual_payment_date")).date());
        })
        .await?;
        create_sat(manager, "bse_financial_results", |t| {
            t.col(ColumnDef::new(Alias::new("scripcode")).text());
            t.col(ColumnDef::new(Alias::new("audited_unaudited")).text());
            t.col(ColumnDef::new(Alias::new("standalone_consolidated")).text());
            t.col(ColumnDef::new(Alias::new("period_start_date")).date());
            t.col(ColumnDef::new(Alias::new("period_end_date")).date());
            t.col(ColumnDef::new(Alias::new("ind_as")).text());
        })
        .await?;
        create_sat(manager, "bse_insider_trading", |t| {
            t.col(ColumnDef::new(Alias::new("scripcode")).text());
            t.col(ColumnDef::new(Alias::new("type_of_security")).text());
        })
        .await?;
        create_sat(manager, "bse_shareholding_pattern", |t| {
            t.col(ColumnDef::new(Alias::new("scripcode")).text());
            t.col(ColumnDef::new(Alias::new("as_on_date")).date());
            t.col(ColumnDef::new(Alias::new("promoter_and_group")).text());
            t.col(ColumnDef::new(Alias::new("public_val")).text());
            t.col(ColumnDef::new(Alias::new("emptr")).text());
            t.col(ColumnDef::new(Alias::new("status")).text());
            t.col(ColumnDef::new(Alias::new("submission_date")).date());
            t.col(ColumnDef::new(Alias::new("revised_filing_date")).date());
        })
        .await?;
        create_sat(manager, "bse_voting_results", |t| {
            t.col(ColumnDef::new(Alias::new("scripcode")).text());
            t.col(ColumnDef::new(Alias::new("meeting_date")).date());
            t.col(ColumnDef::new(Alias::new("meeting_type")).text());
        })
        .await?;

        exec(
            manager,
            "INSERT INTO bse_announcements (item_id, scripcode) SELECT id, scripcode FROM bse_rss_items WHERE feed_kind = 'announcements'",
        )
        .await?;
        exec(
            manager,
            "INSERT INTO bse_annual_reports (item_id, scripcode, as_on_date) SELECT id, scripcode, as_on_date FROM bse_rss_items WHERE feed_kind = 'annual-reports'",
        )
        .await?;
        exec(
            manager,
            "INSERT INTO bse_board_meetings (item_id, scripcode, meeting_date, purpose) SELECT id, scripcode, meeting_date, purpose FROM bse_rss_items WHERE feed_kind = 'board-meetings'",
        )
        .await?;
        exec(
            manager,
            "INSERT INTO bse_corporate_actions (item_id, scripcode, segment, purpose, rd_date, bc_start_date, bc_end_date, nd_start_date, nd_end_date, actual_payment_date) SELECT id, scripcode, segment, purpose, rd_date, bc_start_date, bc_end_date, nd_start_date, nd_end_date, actual_payment_date FROM bse_rss_items WHERE feed_kind = 'corporate-actions'",
        )
        .await?;
        exec(
            manager,
            "INSERT INTO bse_financial_results (item_id, scripcode, audited_unaudited, standalone_consolidated, period_start_date, period_end_date, ind_as) SELECT id, scripcode, audited_unaudited, standalone_consolidated, period_start_date, period_end_date, ind_as FROM bse_rss_items WHERE feed_kind = 'financial-results'",
        )
        .await?;
        exec(
            manager,
            "INSERT INTO bse_insider_trading (item_id, scripcode, type_of_security) SELECT id, scripcode, type_of_security FROM bse_rss_items WHERE feed_kind = 'insider-trading'",
        )
        .await?;
        exec(
            manager,
            "INSERT INTO bse_shareholding_pattern (item_id, scripcode, as_on_date, promoter_and_group, public_val, emptr, status, submission_date, revised_filing_date) SELECT id, scripcode, as_on_date, promoter_and_group, public_val, emptr, status, submission_date, revised_filing_date FROM bse_rss_items WHERE feed_kind = 'shareholding-pattern'",
        )
        .await?;
        exec(
            manager,
            "INSERT INTO bse_voting_results (item_id, scripcode, meeting_date, meeting_type) SELECT id, scripcode, meeting_date, meeting_type FROM bse_rss_items WHERE feed_kind = 'voting-results'",
        )
        .await?;

        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::drop_gin_index(db, backend, "bse_rss_items_scripcode_trgm_idx").await?;
        for (name, table, col) in SAT_TRGM {
            trigram::create_gin_index(db, backend, name, table, col).await?;
        }

        for col in TEXT_COLS.iter().chain(DATE_COLS.iter()) {
            drop_parent_col(manager, col).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in TEXT_COLS {
            add_parent_text(manager, col).await?;
        }
        for col in DATE_COLS {
            add_parent_date(manager, col).await?;
        }

        exec(
            manager,
            "UPDATE bse_rss_items SET scripcode = (SELECT scripcode FROM bse_announcements s WHERE s.item_id = bse_rss_items.id) WHERE feed_kind = 'announcements'",
        )
        .await?;
        exec(
            manager,
            "UPDATE bse_rss_items SET scripcode = (SELECT scripcode FROM bse_annual_reports s WHERE s.item_id = bse_rss_items.id), as_on_date = (SELECT as_on_date FROM bse_annual_reports s WHERE s.item_id = bse_rss_items.id) WHERE feed_kind = 'annual-reports'",
        )
        .await?;
        exec(
            manager,
            "UPDATE bse_rss_items SET scripcode = (SELECT scripcode FROM bse_board_meetings s WHERE s.item_id = bse_rss_items.id), meeting_date = (SELECT meeting_date FROM bse_board_meetings s WHERE s.item_id = bse_rss_items.id), purpose = (SELECT purpose FROM bse_board_meetings s WHERE s.item_id = bse_rss_items.id) WHERE feed_kind = 'board-meetings'",
        )
        .await?;
        exec(
            manager,
            "UPDATE bse_rss_items SET scripcode = (SELECT scripcode FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id), segment = (SELECT segment FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id), purpose = (SELECT purpose FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id), rd_date = (SELECT rd_date FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id), bc_start_date = (SELECT bc_start_date FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id), bc_end_date = (SELECT bc_end_date FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id), nd_start_date = (SELECT nd_start_date FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id), nd_end_date = (SELECT nd_end_date FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id), actual_payment_date = (SELECT actual_payment_date FROM bse_corporate_actions s WHERE s.item_id = bse_rss_items.id) WHERE feed_kind = 'corporate-actions'",
        )
        .await?;
        exec(
            manager,
            "UPDATE bse_rss_items SET scripcode = (SELECT scripcode FROM bse_financial_results s WHERE s.item_id = bse_rss_items.id), audited_unaudited = (SELECT audited_unaudited FROM bse_financial_results s WHERE s.item_id = bse_rss_items.id), standalone_consolidated = (SELECT standalone_consolidated FROM bse_financial_results s WHERE s.item_id = bse_rss_items.id), period_start_date = (SELECT period_start_date FROM bse_financial_results s WHERE s.item_id = bse_rss_items.id), period_end_date = (SELECT period_end_date FROM bse_financial_results s WHERE s.item_id = bse_rss_items.id), ind_as = (SELECT ind_as FROM bse_financial_results s WHERE s.item_id = bse_rss_items.id) WHERE feed_kind = 'financial-results'",
        )
        .await?;
        exec(
            manager,
            "UPDATE bse_rss_items SET scripcode = (SELECT scripcode FROM bse_insider_trading s WHERE s.item_id = bse_rss_items.id), type_of_security = (SELECT type_of_security FROM bse_insider_trading s WHERE s.item_id = bse_rss_items.id) WHERE feed_kind = 'insider-trading'",
        )
        .await?;
        exec(
            manager,
            "UPDATE bse_rss_items SET scripcode = (SELECT scripcode FROM bse_shareholding_pattern s WHERE s.item_id = bse_rss_items.id), as_on_date = (SELECT as_on_date FROM bse_shareholding_pattern s WHERE s.item_id = bse_rss_items.id), promoter_and_group = (SELECT promoter_and_group FROM bse_shareholding_pattern s WHERE s.item_id = bse_rss_items.id), public_val = (SELECT public_val FROM bse_shareholding_pattern s WHERE s.item_id = bse_rss_items.id), emptr = (SELECT emptr FROM bse_shareholding_pattern s WHERE s.item_id = bse_rss_items.id), status = (SELECT status FROM bse_shareholding_pattern s WHERE s.item_id = bse_rss_items.id), submission_date = (SELECT submission_date FROM bse_shareholding_pattern s WHERE s.item_id = bse_rss_items.id), revised_filing_date = (SELECT revised_filing_date FROM bse_shareholding_pattern s WHERE s.item_id = bse_rss_items.id) WHERE feed_kind = 'shareholding-pattern'",
        )
        .await?;
        exec(
            manager,
            "UPDATE bse_rss_items SET scripcode = (SELECT scripcode FROM bse_voting_results s WHERE s.item_id = bse_rss_items.id), meeting_date = (SELECT meeting_date FROM bse_voting_results s WHERE s.item_id = bse_rss_items.id), meeting_type = (SELECT meeting_type FROM bse_voting_results s WHERE s.item_id = bse_rss_items.id) WHERE feed_kind = 'voting-results'",
        )
        .await?;

        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::create_gin_index(
            db,
            backend,
            "bse_rss_items_scripcode_trgm_idx",
            "bse_rss_items",
            "scripcode",
        )
        .await?;

        for table in SAT_TABLES {
            manager
                .drop_table(Table::drop().table(Alias::new(*table)).to_owned())
                .await?;
        }
        Ok(())
    }
}
