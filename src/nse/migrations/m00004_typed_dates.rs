use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, Value};
use sea_orm_migration::prelude::*;

use super::super::description::{parse_nse_date, parse_nse_datetime};

#[derive(DeriveMigrationName)]
pub struct Migration;

enum ColumnKind {
    Date,
    DateTime,
}

async fn column_type(
    manager: &SchemaManager<'_>,
    table: &str,
    column: &str,
) -> Result<String, DbErr> {
    let conn = manager.get_connection();
    let backend = manager.get_database_backend();
    match backend {
        DatabaseBackend::Postgres => {
            let sql = format!(
                "SELECT data_type FROM information_schema.columns \
                 WHERE table_name = '{table}' AND column_name = '{column}'"
            );
            let row = conn.query_one(Statement::from_string(backend, sql)).await?;
            Ok(row
                .and_then(|r| r.try_get_by_index::<String>(0).ok())
                .unwrap_or_default())
        }
        DatabaseBackend::Sqlite => {
            let sql = format!("PRAGMA table_info({table})");
            let rows = conn.query_all(Statement::from_string(backend, sql)).await?;
            for row in rows {
                let name: String = row.try_get_by_index(1).unwrap_or_default();
                if name == column {
                    return Ok(row.try_get_by_index::<String>(2).unwrap_or_default());
                }
            }
            Ok(String::new())
        }
        _ => Ok(String::new()),
    }
}

fn is_text_type(t: &str) -> bool {
    let t = t.to_ascii_lowercase();
    t == "text" || t.contains("char") || t == "varchar"
}

async fn exec(manager: &SchemaManager<'_>, sql: String) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute(Statement::from_string(manager.get_database_backend(), sql))
        .await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        convert_column(manager, "nse_rss_items", "pub_date", ColumnKind::DateTime).await?;
        convert_column(
            manager,
            "nse_rss_items",
            "original_submission_date",
            ColumnKind::DateTime,
        )
        .await?;
        for col in [
            "as_on_date",
            "record_date",
            "book_closure_start_date",
            "book_closure_end_date",
            "period_ended",
            "for_quarter_ending",
            "period_end_date",
            "meeting_date",
        ] {
            convert_column(manager, "nse_rss_items", col, ColumnKind::Date).await?;
        }
        convert_column(
            manager,
            "nse_feed_status",
            "last_build_date",
            ColumnKind::DateTime,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager;
        Ok(())
    }
}

async fn convert_column(
    manager: &SchemaManager<'_>,
    table: &str,
    column: &str,
    kind: ColumnKind,
) -> Result<(), DbErr> {
    let ty = column_type(manager, table, column).await?;
    if ty.is_empty() || !is_text_type(&ty) {
        return Ok(());
    }
    let backend = manager.get_database_backend();
    if backend == DatabaseBackend::Postgres && column == "pub_date" {
        exec(
            manager,
            "ALTER TABLE nse_rss_items ALTER COLUMN pub_date DROP NOT NULL".into(),
        )
        .await?;
    }
    let conn = manager.get_connection();
    let pk = if table == "nse_rss_items" {
        "id"
    } else {
        "feed_kind"
    };
    let rows = conn
        .query_all(Statement::from_string(
            backend,
            format!("SELECT {pk}, {column} FROM {table}"),
        ))
        .await?;
    for row in rows {
        let raw: Option<String> = row.try_get_by_index::<String>(1).ok();
        let iso = raw.as_deref().and_then(|raw| match kind {
            ColumnKind::Date => parse_nse_date(raw).map(|d| d.format("%Y-%m-%d").to_string()),
            ColumnKind::DateTime => parse_nse_datetime(raw).map(|d| d.to_rfc3339()),
        });
        let pk_val = if table == "nse_rss_items" {
            Value::BigInt(Some(row.try_get_by_index::<i64>(0)?))
        } else {
            Value::from(row.try_get_by_index::<String>(0)?)
        };
        conn.execute(Statement::from_sql_and_values(
            backend,
            format!("UPDATE {table} SET {column} = $1 WHERE {pk} = $2"),
            [iso.into(), pk_val],
        ))
        .await?;
    }
    match backend {
        DatabaseBackend::Postgres => {
            let sql_type = match kind {
                ColumnKind::Date => "date",
                ColumnKind::DateTime => "timestamptz",
            };
            exec(
                manager,
                format!(
                    "ALTER TABLE {table} ALTER COLUMN {column} TYPE {sql_type} \
                     USING {column}::{sql_type}"
                ),
            )
            .await?;
        }
        DatabaseBackend::Sqlite => {
            let mut def = ColumnDef::new(Alias::new(column));
            match kind {
                ColumnKind::Date => {
                    def.date().null();
                }
                ColumnKind::DateTime => {
                    def.timestamp_with_time_zone().null();
                }
            }
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new(table))
                        .modify_column(def)
                        .to_owned(),
                )
                .await?;
        }
        _ => {}
    }
    Ok(())
}
