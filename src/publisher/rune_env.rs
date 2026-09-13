//! Rune sandbox bindings for subscriber mailing.

use std::sync::Arc;

use lariv_rs::plugins::filesystem::node;
use lariv_rs::rune_env::{
    NativeBinding, RuneEnvCapability, RuneEnvCtx, RuneEnvRegistrar, block_on_async, json_to_rune,
    rune_to_json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;
use serde_json::{Map, Value as JsonValue};

use super::email::{
    email_template_context_schema_json, load_attachments, matching_subscriber_emails,
    send_subscriber_emails, template_context_specs, validate_template_context,
};
use super::entities::subscriber::{
    self, Entity as SubscriberEntity, NewsletterInterval, UniqueFilter, json_to_optional_string_list,
};
use super::preferences::load_preferences;

/// Registers subscriber mailing helpers onto the assistant Rune environment.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl RuneEnvRegistrar for Hook {
    fn register_rune_env(self, rune_env: &mut RuneEnvCapability) {
        rune_env.register_contextual(
            "email_template_context_schema",
            "email_template_context_schema(()) -> #{ type: string, additionalProperties: bool, required: [string], properties: object, signature: string }  // exact `context` object for send_email_to_subscribers; keys come from {{placeholders}} in the Publisher HTML template (`email` / `subscriber_email` are filled per recipient)",
            |_ctx| NativeBinding::Function(Arc::new(email_template_context_schema)),
        );
        rune_env.register_contextual(
            "send_email_to_subscribers",
            "send_email_to_subscribers(#{ subject: string, interval: \"daily\" | \"weekly\" | \"monthly\", filter: #{ exchanges?: [string], companies?: [string], event_types?: [string] }, attachments?: [int | #{ id: int } | #{ path: string }], context?: object }) -> #{ sent: int, failed: int, failures: [#{ email: string, error: string }] }  // render the Publisher HTML template; `context` must match email_template_context_schema(()) exactly; recipients are filtered by newsletter interval and UniqueFilter; optional VNode attachments",
            |_ctx| NativeBinding::Function(Arc::new(send_email_to_subscribers)),
        );
        rune_env.register_contextual(
            "get_unique_filters",
            "get_unique_filters(interval: \"daily\" | \"weekly\" | \"monthly\" | #{ interval: \"daily\" | \"weekly\" | \"monthly\" }) -> [#{ exchanges: [string] | (), companies: [string] | (), event_types: [string] | () }]  // distinct unique subscriber filter configurations for the given newsletter interval",
            |_ctx| NativeBinding::Function(Arc::new(get_unique_filters)),
        );
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SendArgs {
    subject: String,
    interval: NewsletterIntervalArg,
    #[serde(alias = "unique_filter")]
    filter: UniqueFilterArg,
    #[serde(default)]
    attachments: Vec<AttachmentArg>,
    #[serde(default)]
    context: Map<String, JsonValue>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NewsletterIntervalArg {
    Interval(NewsletterInterval),
    Str(String),
}

impl NewsletterIntervalArg {
    fn into_interval(self) -> Result<NewsletterInterval, String> {
        match self {
            Self::Interval(i) => Ok(i),
            Self::Str(s) => NewsletterInterval::parse(&s).ok_or_else(|| {
                format!(
                    "invalid newsletter interval {s:?}, expected 'daily', 'weekly', or 'monthly'"
                )
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UniqueFilterArg {
    #[serde(default)]
    exchanges: Option<Vec<String>>,
    #[serde(default)]
    companies: Option<Vec<String>>,
    #[serde(default)]
    event_types: Option<Vec<String>>,
}

impl UniqueFilterArg {
    fn into_unique_filter(self) -> UniqueFilter {
        UniqueFilter {
            exchanges: self.exchanges.filter(|v| !v.is_empty()),
            companies: self.companies.filter(|v| !v.is_empty()),
            event_types: self.event_types.filter(|v| !v.is_empty()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AttachmentArg {
    Id(i64),
    Ref(AttachmentRef),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttachmentRef {
    #[serde(default)]
    id: Option<i64>,
    #[serde(default)]
    path: Option<String>,
}

fn parse_send_email_args(args: &[rune::Value]) -> Result<SendArgs, String> {
    if args.len() != 1 {
        return Err("send_email_to_subscribers requires exactly one object argument".into());
    }
    let value = rune_to_json(&args[0])?;
    if !value.is_object() {
        return Err("send_email_to_subscribers requires an object argument".into());
    }
    serde_json::from_value(value)
        .map_err(|e| format!("invalid send_email_to_subscribers arguments: {e}"))
}

fn parse_unit_arg(fn_name: &str, args: &[rune::Value]) -> Result<(), String> {
    if args.len() != 1 {
        return Err(format!("{fn_name} requires exactly one argument: ()"));
    }
    match rune_to_json(&args[0])? {
        JsonValue::Null => Ok(()),
        other => Err(format!("{fn_name} requires (), got {other}")),
    }
}

fn email_template_context_schema(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    parse_unit_arg("email_template_context_schema", args)?;
    let db = ctx.db.clone();
    let out = block_on_async(async move {
        let prefs = load_preferences(&db).await.map_err(|e| e.to_string())?;
        if prefs.html_template.trim().is_empty() {
            return Err("Set an HTML template in Publisher preferences first.".to_string());
        }
        Ok::<_, String>(email_template_context_schema_json(&prefs.html_template))
    })?;
    json_to_rune(out)
}

fn send_email_to_subscribers(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed = parse_send_email_args(args)?;
    let subject = parsed.subject.trim().to_string();
    if subject.is_empty() {
        return Err("subject is required".into());
    }
    let interval = parsed.interval.into_interval()?;
    let filter = parsed.filter.into_unique_filter();
    let db = ctx.db.clone();
    let store = Arc::clone(&ctx.store);
    let out = block_on_async(async move {
        let prefs = load_preferences(&db).await.map_err(|e| e.to_string())?;
        if prefs.html_template.trim().is_empty() {
            return Err(
                "Set an HTML template in Publisher preferences before sending.".to_string(),
            );
        }
        let specs = template_context_specs(&prefs.html_template, "");
        validate_template_context(&parsed.context, &specs)?;
        let recipients = matching_subscriber_emails(&db, interval, &filter)
            .await
            .map_err(|e| e.to_string())?;
        if recipients.is_empty() {
            return Err(format!(
                "There are no subscribers matching interval {interval} and the provided filter."
            ));
        }
        let attachment_ids = resolve_attachment_ids(&db, &parsed.attachments).await?;
        let attachments = load_attachments(&db, store.as_ref(), &attachment_ids)
            .await
            .map_err(|e| e.to_string())?;
        let report = send_subscriber_emails(
            &prefs,
            &recipients,
            &subject,
            &prefs.html_template,
            &JsonValue::Object(parsed.context),
            attachments,
        )
        .await
        .map_err(|e| e.to_string())?;
        Ok::<_, String>(report.to_json())
    })?;
    json_to_rune(out)
}

async fn resolve_attachment_ids(
    db: &sea_orm::DatabaseConnection,
    refs: &[AttachmentArg],
) -> Result<Vec<i64>, String> {
    let mut ids = Vec::new();
    for (i, item) in refs.iter().enumerate() {
        match item {
            AttachmentArg::Id(id) if *id > 0 => ids.push(*id),
            AttachmentArg::Id(id) => {
                return Err(format!(
                    "attachments[{i}] id must be a positive VNode id, got {id}"
                ));
            }
            AttachmentArg::Ref(AttachmentRef { id: Some(id), path })
                if path.as_deref().is_none_or(str::is_empty) =>
            {
                if *id <= 0 {
                    return Err(format!(
                        "attachments[{i}] id must be a positive VNode id, got {id}"
                    ));
                }
                ids.push(*id);
            }
            AttachmentArg::Ref(AttachmentRef {
                id: None,
                path: Some(path),
            }) => {
                let path = path.trim();
                if path.is_empty() {
                    return Err(format!("attachments[{i}] path is empty"));
                }
                let (vnode, norm) = node::get_by_path(db, path)
                    .await
                    .map_err(|e| e.to_string())?;
                let Some(vnode) = vnode else {
                    return Err(format!("attachment not found at path \"{path}\""));
                };
                if norm == "/" {
                    return Err("attachments cannot be the filesystem root".into());
                }
                ids.push(vnode.id);
            }
            AttachmentArg::Ref(AttachmentRef {
                id: Some(_),
                path: Some(path),
            }) if !path.trim().is_empty() => {
                return Err(format!(
                    "attachments[{i}]: provide either path or id, not both"
                ));
            }
            AttachmentArg::Ref(_) => {
                return Err(format!("attachments[{i}]: path or id is required"));
            }
        }
    }
    Ok(ids)
}

fn parse_interval_arg(args: &[rune::Value]) -> Result<NewsletterInterval, String> {
    if args.len() != 1 {
        return Err("get_unique_filters requires exactly one argument".into());
    }
    let val = rune_to_json(&args[0])?;
    let interval_str = match &val {
        JsonValue::String(s) => s.as_str(),
        JsonValue::Object(map) => map
            .get("interval")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing or non-string 'interval' field in argument object".to_string())?,
        other => {
            return Err(format!(
                "get_unique_filters requires a string interval or object with 'interval', got {other}"
            ));
        }
    };
    NewsletterInterval::parse(interval_str).ok_or_else(|| {
        format!(
            "invalid newsletter interval {interval_str:?}, expected 'daily', 'weekly', or 'monthly'"
        )
    })
}

fn get_unique_filters(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let interval = parse_interval_arg(args)?;
    let db = ctx.db.clone();
    let out = block_on_async(async move {
        let rows = SubscriberEntity::find()
            .filter(subscriber::Column::NewsletterInterval.eq(interval))
            .order_by_asc(subscriber::Column::Id)
            .all(&db)
            .await
            .map_err(|e| e.to_string())?;

        let mut seen = std::collections::HashSet::new();
        let mut unique_filters = Vec::new();
        for row in rows {
            let filter = UniqueFilter {
                exchanges: json_to_optional_string_list(&row.filter_exchanges),
                companies: json_to_optional_string_list(&row.filter_entities),
                event_types: json_to_optional_string_list(&row.filter_event_types),
            };
            if seen.insert(filter.clone()) {
                unique_filters.push(filter);
            }
        }
        serde_json::to_value(&unique_filters).map_err(|e| e.to_string())
    })?;
    json_to_rune(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use lariv_rs::plugins::filesystem::storage::{DynFilestore, UnimplementedFilestore};
    use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
    use sea_orm_migration::MigratorTrait;
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

    fn call_fn(name: &str, db: &DatabaseConnection, args: &[rune::Value]) -> Result<rune::Value, String> {
        let cap = registered_env();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, f)| f)
            .unwrap_or_else(|| panic!("{name}"));
        f(&env_ctx, args)
    }

    fn call_err(name: &str, args: &[rune::Value]) -> String {
        let db = DatabaseConnection::default();
        call_fn(name, &db, args).expect_err("expected error")
    }

    fn json_arg(value: JsonValue) -> rune::Value {
        json_to_rune(value).expect("args")
    }

    #[test]
    fn registers_mailing_helpers() {
        let names = registered_env().all_names();
        assert!(
            names.iter().any(|name| name == "send_email_to_subscribers"),
            "expected send_email_to_subscribers in {names:?}"
        );
        assert!(
            names
                .iter()
                .any(|name| name == "email_template_context_schema"),
            "expected email_template_context_schema in {names:?}"
        );
        assert!(
            names.iter().any(|name| name == "get_unique_filters"),
            "expected get_unique_filters in {names:?}"
        );
    }

    #[test]
    fn parse_interval_arg_accepts_strings_and_objects() {
        assert_eq!(
            parse_interval_arg(&[json_arg(json!("daily"))]).unwrap(),
            NewsletterInterval::Daily
        );
        assert_eq!(
            parse_interval_arg(&[json_arg(json!("Weekly"))]).unwrap(),
            NewsletterInterval::Weekly
        );
        assert_eq!(
            parse_interval_arg(&[json_arg(json!("MONTHLY"))]).unwrap(),
            NewsletterInterval::Monthly
        );
        assert_eq!(
            parse_interval_arg(&[json_arg(json!({ "interval": "daily" }))]).unwrap(),
            NewsletterInterval::Daily
        );
        assert_eq!(
            parse_interval_arg(&[json_arg(json!({ "interval": "Weekly" }))]).unwrap(),
            NewsletterInterval::Weekly
        );
    }

    #[test]
    fn parse_interval_arg_rejects_invalid_inputs() {
        let err = parse_interval_arg(&[]).unwrap_err();
        assert!(err.contains("exactly one argument"), "{err}");

        let err = parse_interval_arg(&[json_arg(json!("daily")), json_arg(json!("weekly"))]).unwrap_err();
        assert!(err.contains("exactly one argument"), "{err}");

        let err = parse_interval_arg(&[json_arg(json!("yearly"))]).unwrap_err();
        assert!(err.contains("invalid newsletter interval"), "{err}");

        let err = parse_interval_arg(&[json_arg(json!({ "interval": "yearly" }))]).unwrap_err();
        assert!(err.contains("invalid newsletter interval"), "{err}");

        let err = parse_interval_arg(&[json_arg(json!(123))]).unwrap_err();
        assert!(err.contains("requires a string interval or object"), "{err}");

        let err = parse_interval_arg(&[json_arg(json!({}))]).unwrap_err();
        assert!(err.contains("missing or non-string 'interval' field"), "{err}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_unique_filters_returns_distinct_filters() {
        let mut opt = sea_orm::ConnectOptions::new("sqlite::memory:");
        opt.max_connections(5);
        let db = sea_orm::Database::connect(opt)
            .await
            .expect("db connect");
        crate::publisher::migrations::Migrator::up(&db, None)
            .await
            .expect("migrations up");

        let now = Utc::now();

        // 1. Daily subscriber with exchange + company
        subscriber::ActiveModel {
            email: Set("user1@example.com".into()),
            subscription_date: Set(now),
            filter_exchanges: Set(Some(json!(["nse"]))),
            filter_entities: Set(Some(json!(["RELIANCE"]))),
            filter_event_types: Set(None),
            newsletter_interval: Set(NewsletterInterval::Daily),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("insert sub 1");

        // 2. Another Daily subscriber with the same filters (duplicate)
        subscriber::ActiveModel {
            email: Set("user2@example.com".into()),
            subscription_date: Set(now),
            filter_exchanges: Set(Some(json!(["nse"]))),
            filter_entities: Set(Some(json!(["RELIANCE"]))),
            filter_event_types: Set(None),
            newsletter_interval: Set(NewsletterInterval::Daily),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("insert sub 2");

        // 3. Daily subscriber with different filters
        subscriber::ActiveModel {
            email: Set("user3@example.com".into()),
            subscription_date: Set(now),
            filter_exchanges: Set(None),
            filter_entities: Set(Some(json!(["TCS"]))),
            filter_event_types: Set(Some(json!(["financial-results"]))),
            newsletter_interval: Set(NewsletterInterval::Daily),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("insert sub 3");

        // 4. Weekly subscriber (should not appear in daily results)
        subscriber::ActiveModel {
            email: Set("user4@example.com".into()),
            subscription_date: Set(now),
            filter_exchanges: Set(Some(json!(["bse"]))),
            filter_entities: Set(Some(json!(["INFY"]))),
            filter_event_types: Set(None),
            newsletter_interval: Set(NewsletterInterval::Weekly),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("insert sub 4");

        // Call get_unique_filters("daily")
        let res = call_fn("get_unique_filters", &db, &[json_arg(json!("daily"))]).expect("call");
        let val = rune_to_json(&res).expect("to json");
        let filters: Vec<UniqueFilter> = serde_json::from_value(val).expect("parse unique filters");

        assert_eq!(filters.len(), 2);
        assert_eq!(
            filters[0],
            UniqueFilter {
                exchanges: Some(vec!["nse".into()]),
                companies: Some(vec!["RELIANCE".into()]),
                event_types: None,
            }
        );
        assert_eq!(
            filters[1],
            UniqueFilter {
                exchanges: None,
                companies: Some(vec!["TCS".into()]),
                event_types: Some(vec!["financial-results".into()]),
            }
        );

        // Call get_unique_filters(#{ "interval": "weekly" })
        let res_weekly = call_fn(
            "get_unique_filters",
            &db,
            &[json_arg(json!({ "interval": "weekly" }))],
        )
        .expect("call weekly");
        let val_weekly = rune_to_json(&res_weekly).expect("to json");
        let filters_weekly: Vec<UniqueFilter> =
            serde_json::from_value(val_weekly).expect("parse weekly filters");
        assert_eq!(filters_weekly.len(), 1);
        assert_eq!(
            filters_weekly[0],
            UniqueFilter {
                exchanges: Some(vec!["bse".into()]),
                companies: Some(vec!["INFY".into()]),
                event_types: None,
            }
        );

        // Verify matching_subscriber_emails finds matching recipients
        let match_1 = matching_subscriber_emails(&db, NewsletterInterval::Daily, &filters[0])
            .await
            .expect("matching sub 1");
        assert_eq!(match_1, vec!["user1@example.com", "user2@example.com"]);

        let match_2 = matching_subscriber_emails(&db, NewsletterInterval::Daily, &filters[1])
            .await
            .expect("matching sub 2");
        assert_eq!(match_2, vec!["user3@example.com"]);

        let match_weekly = matching_subscriber_emails(&db, NewsletterInterval::Weekly, &filters_weekly[0])
            .await
            .expect("matching weekly");
        assert_eq!(match_weekly, vec!["user4@example.com"]);

        // Call get_unique_filters("monthly") -> empty list
        let res_monthly = call_fn("get_unique_filters", &db, &[json_arg(json!("monthly"))])
            .expect("call monthly");
        let val_monthly = rune_to_json(&res_monthly).expect("to json");
        let filters_monthly: Vec<UniqueFilter> =
            serde_json::from_value(val_monthly).expect("parse monthly filters");
        assert!(filters_monthly.is_empty());
    }

    #[test]
    fn send_email_to_subscribers_rejects_empty_subject() {
        let err = parse_send_email_args(&[json_arg(json!({}))]).unwrap_err();
        assert!(err.contains("subject") || err.contains("interval") || err.contains("filter"), "{err}");
        let parsed = parse_send_email_args(&[json_arg(json!({
            "subject": "  ",
            "interval": "daily",
            "filter": {}
        }))]).unwrap();
        assert!(parsed.subject.trim().is_empty());
        let err = call_err(
            "send_email_to_subscribers",
            &[json_arg(json!({
                "subject": "  ",
                "interval": "daily",
                "filter": {}
            }))],
        );
        assert!(err.contains("subject"), "{err}");
    }

    #[test]
    fn send_email_to_subscribers_rejects_missing_interval_or_filter() {
        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "filter": {}
        }))]).unwrap_err();
        assert!(err.contains("interval"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "interval": "daily"
        }))]).unwrap_err();
        assert!(err.contains("filter"), "{err}");

        let parsed = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "interval": "yearly",
            "filter": {}
        }))]).unwrap();
        assert!(parsed.interval.into_interval().is_err());
    }

    #[test]
    fn send_email_to_subscribers_rejects_schema_mismatch() {
        let err = parse_send_email_args(&[]).unwrap_err();
        assert!(err.contains("exactly one"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!("hello"))]).unwrap_err();
        assert!(err.contains("object"), "{err}");

        let a = json_arg(json!({ "subject": "Hi", "interval": "daily", "filter": {} }));
        let b = json_arg(json!({ "subject": "Hi", "interval": "daily", "filter": {} }));
        let err = parse_send_email_args(&[a, b]).unwrap_err();
        assert!(err.contains("exactly one"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "interval": "daily",
            "filter": {},
            "nope": true
        }))])
        .unwrap_err();
        assert!(err.contains("unknown field"), "{err}");
        assert!(err.contains("nope"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": 1,
            "interval": "daily",
            "filter": {}
        }))]).unwrap_err();
        assert!(err.contains("invalid"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "interval": "daily",
            "filter": {},
            "attachments": "file.pdf"
        }))])
        .unwrap_err();
        assert!(err.contains("invalid"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "interval": "daily",
            "filter": {},
            "attachments": [{ "id": 1, "extra": true }]
        }))])
        .unwrap_err();
        assert!(err.contains("invalid"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "interval": "daily",
            "filter": {},
            "context": "not-an-object"
        }))])
        .unwrap_err();
        assert!(err.contains("invalid"), "{err}");
    }

    #[test]
    fn send_email_to_subscribers_accepts_exact_schema() {
        let parsed = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "interval": "daily",
            "filter": { "companies": ["RELIANCE"] }
        }))]).unwrap();
        assert_eq!(parsed.subject, "Hi");
        assert_eq!(parsed.interval.into_interval().unwrap(), NewsletterInterval::Daily);
        assert_eq!(
            parsed.filter.into_unique_filter(),
            UniqueFilter {
                exchanges: None,
                companies: Some(vec!["RELIANCE".into()]),
                event_types: None,
            }
        );
        assert!(parsed.attachments.is_empty());
        assert!(parsed.context.is_empty());

        let parsed = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "interval": "weekly",
            "filter": {
                "exchanges": ["nse"],
                "companies": ["TCS"],
                "event_types": ["announcements"]
            },
            "attachments": [3, { "id": 4 }, { "path": "/reports/a.pdf" }],
            "context": { "report_date": "11 Sep 2026" }
        }))])
        .unwrap();
        assert_eq!(parsed.attachments.len(), 3);
        assert_eq!(parsed.context["report_date"], json!("11 Sep 2026"));
    }

    #[test]
    fn send_email_to_subscribers_rejects_missing_object() {
        let err = call_err("send_email_to_subscribers", &[]);
        assert!(
            err.contains("object argument") || err.contains("exactly one"),
            "{err}"
        );
    }

    #[test]
    fn email_template_context_schema_requires_unit() {
        let err = call_err("email_template_context_schema", &[]);
        assert!(err.contains("()"), "{err}");
        let err = call_err("email_template_context_schema", &[json_arg(json!({}))]);
        assert!(err.contains("()"), "{err}");
        let err = call_err(
            "email_template_context_schema",
            &[json_arg(json!(null)), json_arg(json!(null))],
        );
        assert!(err.contains("exactly one"), "{err}");
    }
}
