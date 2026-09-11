//! Rune sandbox bindings for subscriber mailing.

use std::sync::Arc;

use lariv_rs::plugins::filesystem::node;
use lariv_rs::rune_env::{
    NativeBinding, RuneEnvCapability, RuneEnvCtx, RuneEnvRegistrar, block_on_async, json_to_rune,
    rune_to_json,
};
use serde::Deserialize;
use serde_json::{Map, Value as JsonValue};

use super::email::{
    all_subscriber_emails, email_template_context_schema_json, load_attachments,
    send_subscriber_emails, template_context_specs, validate_template_context,
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
            "send_email_to_subscribers(#{ subject: string, attachments?: [int | #{ id: int } | #{ path: string }], context?: object }) -> #{ sent: int, failed: int, failures: [#{ email: string, error: string }] }  // render the Publisher HTML template; `context` must match email_template_context_schema(()) exactly; optional VNode attachments",
            |_ctx| NativeBinding::Function(Arc::new(send_email_to_subscribers)),
        );
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SendArgs {
    subject: String,
    #[serde(default)]
    attachments: Vec<AttachmentArg>,
    #[serde(default)]
    context: Map<String, JsonValue>,
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
        let recipients = all_subscriber_emails(&db)
            .await
            .map_err(|e| e.to_string())?;
        if recipients.is_empty() {
            return Err("There are no subscribers to email.".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
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

    fn call_err(name: &str, args: &[rune::Value]) -> String {
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
        f(&env_ctx, args).expect_err("expected error")
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
    }

    #[test]
    fn send_email_to_subscribers_rejects_empty_subject() {
        let err = parse_send_email_args(&[json_arg(json!({}))]).unwrap_err();
        assert!(err.contains("subject"), "{err}");
        let parsed = parse_send_email_args(&[json_arg(json!({ "subject": "  " }))]).unwrap();
        assert!(parsed.subject.trim().is_empty());
        let err = call_err(
            "send_email_to_subscribers",
            &[json_arg(json!({ "subject": "  " }))],
        );
        assert!(err.contains("subject"), "{err}");
    }

    #[test]
    fn send_email_to_subscribers_rejects_schema_mismatch() {
        let err = parse_send_email_args(&[]).unwrap_err();
        assert!(err.contains("exactly one"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!("hello"))]).unwrap_err();
        assert!(err.contains("object"), "{err}");

        let a = json_arg(json!({ "subject": "Hi" }));
        let b = json_arg(json!({ "subject": "Hi" }));
        let err = parse_send_email_args(&[a, b]).unwrap_err();
        assert!(err.contains("exactly one"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "nope": true
        }))])
        .unwrap_err();
        assert!(err.contains("unknown field"), "{err}");
        assert!(err.contains("nope"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({ "subject": 1 }))]).unwrap_err();
        assert!(err.contains("invalid"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "attachments": "file.pdf"
        }))])
        .unwrap_err();
        assert!(err.contains("invalid"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "attachments": [{ "id": 1, "extra": true }]
        }))])
        .unwrap_err();
        assert!(err.contains("invalid"), "{err}");

        let err = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
            "context": "not-an-object"
        }))])
        .unwrap_err();
        assert!(err.contains("invalid"), "{err}");
    }

    #[test]
    fn send_email_to_subscribers_accepts_exact_schema() {
        let parsed = parse_send_email_args(&[json_arg(json!({ "subject": "Hi" }))]).unwrap();
        assert_eq!(parsed.subject, "Hi");
        assert!(parsed.attachments.is_empty());
        assert!(parsed.context.is_empty());

        let parsed = parse_send_email_args(&[json_arg(json!({
            "subject": "Hi",
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
