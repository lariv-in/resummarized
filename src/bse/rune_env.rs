//! Rune sandbox bindings for BSE RSS trigram search.

use std::sync::Arc;

use lariv_rs::rune_env::{
    NativeBinding, RuneEnvCapability, RuneEnvCtx, RuneEnvRegistrar, block_on_async, json_to_rune,
};

use crate::bse::entities::item::{self, Entity as ItemEntity};
use crate::bse::entities::status::{self, Entity as StatusEntity};
use crate::search::{parse_search_args, results_json, search_models};

const ITEM_TEXT_COLUMNS: &[item::Column] = &[
    item::Column::FeedKind,
    item::Column::Title,
    item::Column::Description,
    item::Column::Scripcode,
    item::Column::MeetingType,
    item::Column::Purpose,
    item::Column::Segment,
    item::Column::TypeOfSecurity,
    item::Column::AuditedUnaudited,
    item::Column::StandaloneConsolidated,
    item::Column::IndAs,
    item::Column::PromoterAndGroup,
    item::Column::PublicVal,
    item::Column::Emptr,
    item::Column::Status,
];

const STATUS_TEXT_COLUMNS: &[status::Column] =
    &[status::Column::FeedKind, status::Column::LastError];

/// Registers BSE table search helpers onto the assistant Rune environment.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl RuneEnvRegistrar for Hook {
    fn register_rune_env(self, rune_env: &mut RuneEnvCapability) {
        rune_env.register_contextual(
            "search_bse_rss_items",
            "search_bse_rss_items(#{ query?: string, from?: string, to?: string, limit?: int }) -> #{ results: [BseRssItem] }  // trigram search across text columns; optional inclusive pub_date range (IST or RFC 3339); at least one of query/from/to",
            |_ctx| NativeBinding::Function(Arc::new(search_bse_rss_items)),
        );
        rune_env.register_contextual(
            "search_bse_feed_status",
            "search_bse_feed_status(#{ query?: string, from?: string, to?: string, limit?: int }) -> #{ results: [BseFeedStatus] }  // trigram search on feed_kind/last_error; optional inclusive last_fetched_at range (IST or RFC 3339); at least one of query/from/to",
            |_ctx| NativeBinding::Function(Arc::new(search_bse_feed_status)),
        );
    }
}

fn search_bse_rss_items(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed = parse_search_args("search_bse_rss_items", args)?;
    let db = ctx.db.clone();
    let rows = block_on_async(async move {
        search_models::<ItemEntity, _, _>(&db, ITEM_TEXT_COLUMNS, item::Column::PubDate, &parsed)
            .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(results_json(&rows))
}

fn search_bse_feed_status(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed = parse_search_args("search_bse_feed_status", args)?;
    let db = ctx.db.clone();
    let rows = block_on_async(async move {
        search_models::<StatusEntity, _, _>(
            &db,
            STATUS_TEXT_COLUMNS,
            status::Column::LastFetchedAt,
            &parsed,
        )
        .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(results_json(&rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use lariv_rs::plugins::filesystem::storage::{DynFilestore, UnimplementedFilestore};
    use sea_orm::DatabaseConnection;
    use serde_json::json;

    fn test_env_ctx<'a>(
        db: &'a DatabaseConnection,
        store: &'a Arc<DynFilestore>,
    ) -> RuneEnvCtx<'a> {
        RuneEnvCtx {
            db,
            store: Arc::clone(store),
            session_id: None,
        }
    }

    fn registered_env() -> RuneEnvCapability {
        let mut cap = RuneEnvCapability::new();
        Hook.register_rune_env(&mut cap);
        cap
    }

    fn call_err(name: &str, args: serde_json::Value) -> String {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, f)| f)
            .unwrap_or_else(|| panic!("{name}"));
        let arg = json_to_rune(args).expect("args");
        f(&env_ctx, &[arg]).expect_err("expected error")
    }

    #[test]
    fn registers_bse_search_bindings() {
        let names = registered_env().all_names();
        for expected in ["search_bse_rss_items", "search_bse_feed_status"] {
            assert!(
                names.iter().any(|name| name == expected),
                "expected {expected} in {names:?}"
            );
        }
    }

    #[test]
    fn search_bse_rss_items_rejects_empty_args() {
        let err = call_err("search_bse_rss_items", json!({}));
        assert!(err.contains("query"), "{err}");
    }

    #[test]
    fn search_bse_feed_status_rejects_invalid_from() {
        let err = call_err("search_bse_feed_status", json!({ "from": "not-a-date" }));
        assert!(err.contains("from"), "{err}");
    }
}
