//! Rune sandbox bindings for BSE RSS trigram search.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use lariv_rs::rune_env::{
    NativeBinding, RuneEnvCapability, RuneEnvCtx, RuneEnvRegistrar, block_on_async, json_to_rune,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::json;

use crate::bse::entities::item::{self, Entity as ItemEntity};
use crate::bse::entities::status::{self, Entity as StatusEntity};
use crate::bse::extras;
use crate::bse::feeds::BseFeedKind;
use crate::search::{compact_json, parse_search_args, results_json, search_models};

const ITEM_TEXT_COLUMNS: &[item::Column] = &[
    item::Column::FeedKind,
    item::Column::Title,
    item::Column::Description,
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
    let payload = block_on_async(async move {
        let core = search_models::<ItemEntity, _, _>(
            &db,
            ITEM_TEXT_COLUMNS,
            item::Column::PubDate,
            &parsed,
        )
        .await?;
        let mut ids: HashSet<i64> = core.iter().map(|row| row.id).collect();
        ids.extend(extras::search_satellite_item_ids(&db, &parsed.query, parsed.limit).await?);
        let id_list: Vec<i64> = ids.into_iter().collect();
        let mut select = ItemEntity::find().filter(item::Column::Id.is_in(id_list));
        if let Some(from) = parsed.from {
            select = select.filter(item::Column::PubDate.gte(from));
        }
        if let Some(to) = parsed.to {
            select = select.filter(item::Column::PubDate.lte(to));
        }
        let rows = select
            .order_by_desc(item::Column::PubDate)
            .limit(parsed.limit)
            .all(&db)
            .await?;
        let mut by_kind: HashMap<String, Vec<i64>> = HashMap::new();
        for row in &rows {
            by_kind
                .entry(row.feed_kind.clone())
                .or_default()
                .push(row.id);
        }
        let mut extras_map = HashMap::new();
        for (slug, kind_ids) in by_kind {
            if let Some(kind) = BseFeedKind::from_slug(&slug) {
                extras_map.extend(extras::load_map(&db, kind, &kind_ids).await?);
            }
        }
        let results: Vec<_> = rows
            .iter()
            .map(|row| {
                let mut value = compact_json(row);
                if let Some(extra) = extras_map.get(&row.id)
                    && let serde_json::Value::Object(map) = &mut value
                {
                    map.insert(
                        extra.json_name().to_string(),
                        compact_json(&extra.to_json()),
                    );
                }
                value
            })
            .collect();
        Ok::<_, sea_orm::DbErr>(json!({ "results": results }))
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(payload)
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
