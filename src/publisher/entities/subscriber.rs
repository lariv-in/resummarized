use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
pub enum NewsletterInterval {
    #[sea_orm(string_value = "daily")]
    Daily,
    #[default]
    #[sea_orm(string_value = "weekly")]
    Weekly,
    #[sea_orm(string_value = "monthly")]
    Monthly,
}

impl NewsletterInterval {
    pub const ALL: &[Self] = &[Self::Daily, Self::Weekly, Self::Monthly];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            "monthly" => Some(Self::Monthly),
            _ => None,
        }
    }
}

impl fmt::Display for NewsletterInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for NewsletterInterval {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("invalid NewsletterInterval: {s:?}"))
    }
}

impl From<NewsletterInterval> for String {
    fn from(v: NewsletterInterval) -> Self {
        v.as_str().into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "subscribers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[sea_orm(unique)]
    pub email: String,
    pub subscription_date: DateTime<Utc>,
    #[sea_orm(column_type = "Json", nullable)]
    pub filter_exchanges: Option<Json>,
    #[sea_orm(column_type = "Json", nullable)]
    pub filter_entities: Option<Json>,
    #[sea_orm(column_type = "Json", nullable)]
    pub filter_event_types: Option<Json>,
    pub newsletter_interval: NewsletterInterval,
    #[sea_orm(column_type = "Uuid", nullable)]
    pub one_time_token: Option<Uuid>,
}

use sea_orm::{ActiveValue::Set, ConnectionTrait};

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert && !self.one_time_token.is_set() {
            self.one_time_token = Set(Some(Uuid::new_v4()));
        }
        Ok(self)
    }
}

/// Trim, split comma-separated tokens, drop empties, and keep first-seen order.
pub fn normalize_filter_list(
    items: impl IntoIterator<Item = impl AsRef<str>>,
) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for item in items {
        for part in item.as_ref().split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !out.iter().any(|existing: &String| existing == trimmed) {
                out.push(trimmed.to_string());
            }
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

/// Like [`normalize_filter_list`], then ASCII-lowercase (exchanges / event types).
pub fn normalize_slug_list(
    items: impl IntoIterator<Item = impl AsRef<str>>,
) -> Option<Vec<String>> {
    let lowered: Vec<String> = items
        .into_iter()
        .map(|s| s.as_ref().to_ascii_lowercase())
        .collect();
    normalize_filter_list(lowered)
}

pub fn string_list_to_json(items: Option<Vec<String>>) -> Option<Json> {
    items.map(|items| JsonValue::Array(items.into_iter().map(JsonValue::String).collect()))
}

pub fn json_to_string_list(value: &Option<Json>) -> Vec<String> {
    match value {
        Some(JsonValue::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        Some(JsonValue::String(s)) => {
            normalize_filter_list(std::iter::once(s.as_str())).unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

pub fn json_to_optional_string_list(value: &Option<Json>) -> Option<Vec<String>> {
    let items = json_to_string_list(value);
    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

pub fn format_filter_display(value: &Option<Json>) -> String {
    let items = json_to_string_list(value);
    if items.is_empty() {
        "All".into()
    } else {
        items.join(", ")
    }
}

/// Unique filter criteria combination for subscribers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniqueFilter {
    pub exchanges: Option<Vec<String>>,
    pub companies: Option<Vec<String>>,
    pub event_types: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::{
        NewsletterInterval, UniqueFilter, json_to_optional_string_list, json_to_string_list,
        normalize_filter_list, normalize_slug_list, string_list_to_json,
    };
    use serde_json::json;

    #[test]
    fn normalize_filter_list_empty_is_none() {
        assert_eq!(normalize_filter_list(["", "  ", ","]), None);
        assert_eq!(normalize_filter_list(Vec::<String>::new()), None);
    }

    #[test]
    fn normalize_filter_list_trims_splits_and_dedupes() {
        assert_eq!(
            normalize_filter_list([" RELIANCE ", "TCS, INFY", "TCS"]),
            Some(vec!["RELIANCE".into(), "TCS".into(), "INFY".into(),])
        );
    }

    #[test]
    fn normalize_slug_list_lowercases() {
        assert_eq!(
            normalize_slug_list(["NSE", "Bse, nasdaq", "nse"]),
            Some(vec!["nse".into(), "bse".into(), "nasdaq".into()])
        );
    }

    #[test]
    fn newsletter_interval_parse() {
        assert_eq!(
            NewsletterInterval::parse(" Weekly "),
            Some(NewsletterInterval::Weekly)
        );
        assert_eq!(
            NewsletterInterval::parse("daily"),
            Some(NewsletterInterval::Daily)
        );
        assert_eq!(
            NewsletterInterval::parse("MONTHLY"),
            Some(NewsletterInterval::Monthly)
        );
        assert_eq!(NewsletterInterval::parse(""), None);
        assert_eq!(NewsletterInterval::parse("yearly"), None);
        assert_eq!(NewsletterInterval::default(), NewsletterInterval::Weekly);
    }

    #[test]
    fn json_roundtrip_string_list() {
        let stored = string_list_to_json(normalize_slug_list(["nse", "bse"]));
        assert_eq!(stored, Some(json!(["nse", "bse"])));
        assert_eq!(
            json_to_string_list(&stored),
            vec!["nse".to_string(), "bse".to_string()]
        );
        assert!(json_to_string_list(&None).is_empty());
        assert!(string_list_to_json(None).is_none());
    }

    #[test]
    fn json_to_optional_string_list_empty_is_none() {
        assert_eq!(json_to_optional_string_list(&None), None);
        assert_eq!(json_to_optional_string_list(&Some(json!([]))), None);
        assert_eq!(
            json_to_optional_string_list(&Some(json!(["nse", "bse"]))),
            Some(vec!["nse".to_string(), "bse".to_string()])
        );
    }

    #[test]
    fn unique_filter_roundtrip_json() {
        let filter = UniqueFilter {
            exchanges: Some(vec!["nse".into()]),
            companies: Some(vec!["RELIANCE".into()]),
            event_types: None,
        };
        let val = serde_json::to_value(&filter).expect("serialize");
        assert_eq!(
            val,
            json!({
                "exchanges": ["nse"],
                "companies": ["RELIANCE"],
                "event_types": null
            })
        );
        let parsed: UniqueFilter = serde_json::from_value(val).expect("deserialize");
        assert_eq!(parsed, filter);
    }
}
