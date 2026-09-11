//! Shared trigram + datetime-range search for BSE/NSE/Nasdaq Rune bindings.

use chrono::{DateTime, Utc};
use lariv_rs::db::trigram;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value as JsonValue, json};

use crate::dates::{is_blank, parse_search_bound};

#[derive(Debug, Deserialize, Default)]
struct SearchArgs {
    #[serde(default)]
    query: String,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    limit: u64,
}

#[derive(Clone, Debug)]
pub struct ParsedSearch {
    pub query: String,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: u64,
}

/// Parse a Rune object argument into [`ParsedSearch`].
pub fn parse_search_args(fn_name: &str, args: &[rune::Value]) -> Result<ParsedSearch, String> {
    let value = args
        .first()
        .ok_or_else(|| format!("{fn_name} requires an object argument"))?;
    parse_search_json(fn_name, lariv_rs::rune_env::rune_to_json(value)?)
}

/// Parse `#{ query?, from?, to?, limit? }`. At least one of query/from/to is required.
pub fn parse_search_json(fn_name: &str, value: JsonValue) -> Result<ParsedSearch, String> {
    let parsed: SearchArgs =
        serde_json::from_value(value).map_err(|e| format!("invalid {fn_name} arguments: {e}"))?;
    let query = parsed.query.trim().to_string();
    let from = parse_optional_bound(fn_name, "from", parsed.from.as_deref(), false)?;
    let to = parse_optional_bound(fn_name, "to", parsed.to.as_deref(), true)?;
    if query.is_empty() && from.is_none() && to.is_none() {
        return Err(format!("{fn_name} requires query, from, or to"));
    }
    if let (Some(from), Some(to)) = (from, to)
        && from > to
    {
        return Err(format!("{fn_name} from must be at or before to"));
    }
    Ok(ParsedSearch {
        query,
        from,
        to,
        limit: trigram::clamp_search_limit(parsed.limit),
    })
}

fn parse_optional_bound(
    fn_name: &str,
    field: &str,
    raw: Option<&str>,
    end_of_day_if_date: bool,
) -> Result<Option<DateTime<Utc>>, String> {
    let Some(raw) = raw.map(str::trim).filter(|s| !is_blank(s)) else {
        return Ok(None);
    };
    parse_search_bound(raw, end_of_day_if_date)
        .map(Some)
        .ok_or_else(|| format!("{fn_name} invalid {field} datetime: {raw}"))
}

/// Trigram search (if `query` is set) plus an inclusive range on `date_column`.
pub async fn search_models<E, TextCol, DateCol>(
    db: &DatabaseConnection,
    text_columns: &[TextCol],
    date_column: DateCol,
    parsed: &ParsedSearch,
) -> Result<Vec<E::Model>, sea_orm::DbErr>
where
    E: EntityTrait,
    TextCol: ColumnTrait + Copy,
    DateCol: ColumnTrait + Copy,
{
    let mut select = E::find();
    if let Some(from) = parsed.from {
        select = select.filter(date_column.gte(from));
    }
    if let Some(to) = parsed.to {
        select = select.filter(date_column.lte(to));
    }
    if parsed.query.is_empty() {
        select = select.order_by_desc(date_column);
    } else {
        select = trigram::apply_text_search(
            select,
            db.get_database_backend(),
            text_columns,
            &parsed.query,
        );
    }
    select.limit(parsed.limit).all(db).await
}

/// Serialize a model, dropping JSON nulls so sparse XBRL rows stay small.
pub fn compact_json<T: Serialize>(value: &T) -> JsonValue {
    let mut v = serde_json::to_value(value).unwrap_or(JsonValue::Null);
    strip_nulls(&mut v);
    v
}

pub fn results_json<T: Serialize>(rows: &[T]) -> JsonValue {
    json!({
        "results": rows.iter().map(compact_json).collect::<Vec<_>>(),
    })
}

fn strip_nulls(v: &mut JsonValue) {
    match v {
        JsonValue::Object(map) => {
            map.retain(|_, val| !val.is_null());
            for val in map.values_mut() {
                strip_nulls(val);
            }
        }
        JsonValue::Array(items) => {
            for item in items {
                strip_nulls(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dates::date_end_ist;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn requires_query_or_datetime() {
        let err = parse_search_json("search_x", json!({})).unwrap_err();
        assert!(err.contains("query"), "{err}");
        assert!(err.contains("from"), "{err}");
        assert!(err.contains("to"), "{err}");
    }

    #[test]
    fn rejects_invalid_from_and_to() {
        let from_err = parse_search_json("search_x", json!({ "from": "not-a-date" })).unwrap_err();
        assert!(from_err.contains("from"), "{from_err}");
        let to_err = parse_search_json("search_x", json!({ "to": "nope" })).unwrap_err();
        assert!(to_err.contains("to"), "{to_err}");
    }

    #[test]
    fn date_only_to_is_end_of_ist_day() {
        let parsed = parse_search_json("search_x", json!({ "to": "2026-09-07" })).unwrap();
        assert_eq!(parsed.to, date_end_ist(date(2026, 9, 7)));
        assert!(parsed.query.is_empty());
        assert_eq!(parsed.limit, trigram::DEFAULT_SEARCH_LIMIT);
    }

    #[test]
    fn rfc3339_from_keeps_utc_offset() {
        let parsed =
            parse_search_json("search_x", json!({ "from": "2026-09-07T00:00:00Z" })).unwrap();
        assert_eq!(
            parsed.from,
            Some(Utc.with_ymd_and_hms(2026, 9, 7, 0, 0, 0).unwrap())
        );
    }

    #[test]
    fn rejects_from_after_to() {
        let err = parse_search_json(
            "search_x",
            json!({ "from": "2026-09-08", "to": "2026-09-07" }),
        )
        .unwrap_err();
        assert!(err.contains("from"), "{err}");
        assert!(err.contains("to"), "{err}");
    }

    #[test]
    fn compact_json_drops_nulls() {
        #[derive(Serialize)]
        struct Row {
            a: Option<i32>,
            b: i32,
        }
        assert_eq!(compact_json(&Row { a: None, b: 1 }), json!({ "b": 1 }));
    }
}
