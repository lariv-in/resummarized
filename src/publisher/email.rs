//! Outbound SMTP for subscriber mailing.

use lariv_rs::plugins::filesystem::{
    entities::VNode, node, storage::DynFilestore, zip::read_file_bytes,
};
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Attachment, Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
    transport::smtp::client::{Tls, TlsParameters},
};
use minijinja::{
    AutoEscape, Environment, Error as MiniError, Output, State, Value as MiniValue,
    escape_formatter,
};
use sea_orm::DatabaseConnection;
use serde_json::{Map, Value as JsonValue, json};

use super::entities::{NewsletterInterval, PublisherPreferences, UniqueFilter};

#[derive(Debug)]
pub enum EmailSendError {
    From(String),
    Send(String),
    Attachment(String),
    Template(String),
}

impl std::fmt::Display for EmailSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::From(e) => write!(f, "invalid from address: {e}"),
            Self::Send(e) => write!(f, "SMTP send failed: {e}"),
            Self::Attachment(e) => write!(f, "{e}"),
            Self::Template(e) => write!(f, "email template: {e}"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SendReport {
    pub sent: usize,
    pub failures: Vec<(String, String)>,
}

impl SendReport {
    pub fn summary(&self) -> String {
        if self.failures.is_empty() {
            return format!("Sent email to {} subscriber(s).", self.sent);
        }
        let failed: Vec<String> = self
            .failures
            .iter()
            .map(|(email, err)| format!("{email}: {err}"))
            .collect();
        format!(
            "Sent {} email(s). {} failed: {}",
            self.sent,
            self.failures.len(),
            failed.join("; ")
        )
    }

    pub fn to_json(&self) -> JsonValue {
        json!({
            "sent": self.sent,
            "failed": self.failures.len(),
            "failures": self.failures.iter().map(|(email, error)| json!({
                "email": email,
                "error": error,
            })).collect::<Vec<_>>(),
        })
    }
}

pub struct PreparedAttachment {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub mime: String,
}

/// Emails of every subscriber row.
pub async fn all_subscriber_emails(db: &DatabaseConnection) -> Result<Vec<String>, sea_orm::DbErr> {
    use sea_orm::EntityTrait;

    use super::entities::SubscriberEntity;

    Ok(SubscriberEntity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|m| m.email)
        .collect())
}

/// Emails of subscribers matching the given newsletter interval and unique filter.
pub async fn matching_subscriber_emails(
    db: &DatabaseConnection,
    interval: NewsletterInterval,
    filter: &UniqueFilter,
) -> Result<Vec<String>, sea_orm::DbErr> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use super::entities::SubscriberEntity;
    use super::entities::subscriber::{self, json_to_optional_string_list};

    let rows: Vec<subscriber::Model> = SubscriberEntity::find()
        .filter(subscriber::Column::NewsletterInterval.eq(interval))
        .all(db)
        .await?;

    let emails = rows
        .into_iter()
        .filter(|row| {
            let row_filter = UniqueFilter {
                exchanges: json_to_optional_string_list(&row.filter_exchanges),
                companies: json_to_optional_string_list(&row.filter_entities),
                event_types: json_to_optional_string_list(&row.filter_event_types),
            };
            &row_filter == filter
        })
        .map(|row| row.email)
        .collect();

    Ok(emails)
}

/// Load file bytes for each selected VNode. Directory nodes attach their direct child files.
pub async fn load_attachments(
    db: &DatabaseConnection,
    store: &DynFilestore,
    vnode_ids: &[i64],
) -> Result<Vec<PreparedAttachment>, EmailSendError> {
    let mut out = Vec::new();
    for id in vnode_ids.iter().copied().filter(|id| *id > 0) {
        let Some(vnode) = node::get_by_id(db, id)
            .await
            .map_err(|e| EmailSendError::Attachment(e.to_string()))?
        else {
            return Err(EmailSendError::Attachment(format!(
                "Attachment VNode #{id} was not found"
            )));
        };
        if vnode.is_directory {
            let children = node::list_children(db, Some(vnode.id), false, "")
                .await
                .map_err(|e| EmailSendError::Attachment(e.to_string()))?;
            for child in children.into_iter().filter(|c| !c.is_directory) {
                out.push(file_attachment(store, child).await?);
            }
        } else {
            out.push(file_attachment(store, vnode).await?);
        }
    }
    Ok(out)
}

async fn file_attachment(
    store: &DynFilestore,
    vnode: VNode,
) -> Result<PreparedAttachment, EmailSendError> {
    let bytes = read_file_bytes(store, &vnode)
        .await
        .map_err(|e| EmailSendError::Attachment(format!("{}: {e}", vnode.name)))?;
    let mime = mime_for_name(&vnode.name);
    Ok(PreparedAttachment {
        filename: vnode.name,
        bytes,
        mime,
    })
}

pub fn mime_for_name(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        "zip" => "application/zip",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn content_type(mime: &str) -> ContentType {
    mime.parse().unwrap_or_else(|_| {
        "application/octet-stream"
            .parse()
            .expect("octet-stream is a valid content type")
    })
}

fn mixed_body(html: &str, attachments: &[PreparedAttachment]) -> MultiPart {
    let html_part = SinglePart::html(html.to_string());
    let mut multipart = MultiPart::mixed().singlepart(html_part);
    for att in attachments {
        let part =
            Attachment::new(att.filename.clone()).body(att.bytes.clone(), content_type(&att.mime));
        multipart = multipart.singlepart(part);
    }
    multipart
}

/// One form field generated from a template `{{placeholder}}` or `{% for %}` source.
#[derive(Debug, Clone)]
pub struct TemplateContextField {
    pub key: String,
    pub label: String,
    pub input_name: String,
    pub value: String,
    pub multiline: bool,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextValueType {
    String { multiline: bool },
    StringArray,
    ObjectArray { fields: Vec<ObjectField> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectField {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateVarSpec {
    pub key: String,
    pub label: String,
    pub value_type: ContextValueType,
}

const RESERVED_CONTEXT_KEYS: &[&str] = &["email", "subscriber_email"];

const MULTILINE_KEY_NEEDLES: &[&str] = &[
    "disclosure",
    "details",
    "description",
    "message",
    "body",
    "content",
    "html",
    "note",
    "summary",
];

pub fn context_input_name(key: &str) -> String {
    format!("ctx[{key}]")
}

fn is_field_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !RESERVED_CONTEXT_KEYS.contains(&key)
}

fn humanize_placeholder(key: &str) -> String {
    let label = key
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if label.is_empty() {
        key.to_string()
    } else {
        label
    }
}

fn is_multiline_placeholder(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    MULTILINE_KEY_NEEDLES
        .iter()
        .any(|needle| k.contains(needle))
}

impl TemplateVarSpec {
    fn is_multiline(&self) -> bool {
        match &self.value_type {
            ContextValueType::String { multiline } => *multiline,
            ContextValueType::StringArray | ContextValueType::ObjectArray { .. } => true,
        }
    }

    fn hint(&self) -> Option<String> {
        match &self.value_type {
            ContextValueType::String { .. } => None,
            ContextValueType::StringArray => Some("JSON array of strings".into()),
            ContextValueType::ObjectArray { fields } => {
                let keys = fields
                    .iter()
                    .map(|f| f.key.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!("JSON array of objects with keys: {keys}"))
            }
        }
    }

    fn signature_part(&self) -> String {
        match &self.value_type {
            ContextValueType::String { .. } => format!("{}: string", self.key),
            ContextValueType::StringArray => format!("{}: [string]", self.key),
            ContextValueType::ObjectArray { fields } => {
                let inner = fields
                    .iter()
                    .map(|f| format!("{}: string", f.key))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}: [#{{ {inner} }}]", self.key)
            }
        }
    }

    fn schema_property(&self) -> JsonValue {
        match &self.value_type {
            ContextValueType::String { multiline } => json!({
                "type": "string",
                "label": self.label,
                "multiline": multiline,
            }),
            ContextValueType::StringArray => json!({
                "type": "array",
                "label": self.label,
                "items": { "type": "string" },
            }),
            ContextValueType::ObjectArray { fields } => {
                let mut properties = Map::new();
                let mut required = Vec::new();
                for field in fields {
                    properties.insert(
                        field.key.clone(),
                        json!({ "type": "string", "label": field.label }),
                    );
                    required.push(json!(field.key));
                }
                json!({
                    "type": "array",
                    "label": self.label,
                    "items": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": required,
                        "properties": properties,
                    },
                })
            }
        }
    }
}

fn jinja_tag_inner(tag: &str) -> &str {
    tag.trim()
        .trim_start_matches("{%")
        .trim_end_matches("%}")
        .trim()
        .trim_start_matches('-')
        .trim_end_matches('-')
        .trim()
}

fn next_jinja_tag(src: &str, from: usize) -> Option<(usize, usize, String)> {
    let rel = src[from..].find("{%")?;
    let start = from + rel;
    let end_rel = src[start..].find("%}")?;
    let end = start + end_rel + 2;
    let inner = jinja_tag_inner(&src[start..end]).to_string();
    Some((start, end, inner))
}

fn parse_for_loops(src: &str) -> Vec<(String, String, String)> {
    let mut loops = Vec::new();
    let mut from = 0;
    while let Some((_start, end, inner)) = next_jinja_tag(src, from) {
        from = end;
        let Some(rest) = inner.strip_prefix("for") else {
            continue;
        };
        let rest = rest.trim();
        let Some((var, source)) = rest.split_once(" in ") else {
            continue;
        };
        let var = var.trim();
        let source = source.trim();
        if !is_field_key(var) || !is_field_key(source) {
            continue;
        }
        let mut depth = 1;
        let body_start = end;
        let mut body_end = src.len();
        let mut cursor = end;
        while let Some((t_start, t_end, t_inner)) = next_jinja_tag(src, cursor) {
            cursor = t_end;
            if t_inner == "endfor" || t_inner.starts_with("endfor ") {
                depth -= 1;
                if depth == 0 {
                    body_end = t_start;
                    from = t_end;
                    break;
                }
            } else if t_inner.starts_with("for ") || t_inner == "for" {
                depth += 1;
            }
        }
        loops.push((
            var.to_string(),
            source.to_string(),
            src[body_start..body_end].to_string(),
        ));
    }
    loops
}

fn expr_from_mustache(inner: &str) -> &str {
    inner
        .trim()
        .trim_start_matches('-')
        .trim_end_matches('-')
        .trim()
        .split('|')
        .next()
        .unwrap_or("")
        .trim()
}

fn dotted_fields_in_body(body: &str, loop_var: &str) -> (bool, Vec<String>) {
    let prefix = format!("{loop_var}.");
    let mut fields = Vec::new();
    let mut saw_bare = false;
    let mut search = body;
    while let Some(i) = search.find("{{") {
        let after = &search[i + 2..];
        let Some(end) = after.find("}}") else {
            break;
        };
        let expr = expr_from_mustache(&after[..end]);
        if expr == loop_var {
            saw_bare = true;
        } else if let Some(field) = expr.strip_prefix(&prefix) {
            let field = field
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect::<String>();
            if is_field_key(&field) && !fields.iter().any(|f| f == &field) {
                fields.push(field);
            }
        }
        search = &after[end + 2..];
    }
    (saw_bare, fields)
}

fn join_filter_sources(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut search = src;
    while let Some(i) = search.find("{{") {
        let after = &search[i + 2..];
        let Some(end) = after.find("}}") else {
            break;
        };
        let inner = after[..end]
            .trim()
            .trim_start_matches('-')
            .trim_end_matches('-')
            .trim();
        if let Some((name, filter)) = inner.split_once('|') {
            let name = name.trim();
            if filter.trim().starts_with("join") && is_field_key(name) {
                out.push(name.to_string());
            }
        }
        search = &after[end + 2..];
    }
    out
}

/// Top-level `{{placeholders}}` in a template, excluding per-subscriber keys.
pub fn template_placeholders(src: &str) -> Vec<String> {
    template_context_specs(src, "")
        .into_iter()
        .map(|s| s.key)
        .collect()
}

pub fn collect_template_placeholders(html: &str, subject: &str) -> Vec<String> {
    template_context_specs(html, subject)
        .into_iter()
        .map(|s| s.key)
        .collect()
}

/// Infer context argument types from `{{placeholders}}`, `| join`, and `{% for %}`.
pub fn template_context_specs(html: &str, subject: &str) -> Vec<TemplateVarSpec> {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| AutoEscape::None);
    let mut keys: Vec<String> = Vec::new();
    for src in [html, subject] {
        if src.is_empty() {
            continue;
        }
        let Ok(tmpl) = env.template_from_str(src) else {
            continue;
        };
        for key in tmpl.undeclared_variables(false) {
            if is_field_key(&key) && !keys.iter().any(|k| k == &key) {
                keys.push(key);
            }
        }
    }
    keys.sort();
    keys.dedup();

    let mut types: std::collections::BTreeMap<String, ContextValueType> = keys
        .iter()
        .map(|key| {
            (
                key.clone(),
                ContextValueType::String {
                    multiline: is_multiline_placeholder(key),
                },
            )
        })
        .collect();

    for name in join_filter_sources(html) {
        if keys.iter().any(|k| k == &name) {
            types.insert(name, ContextValueType::StringArray);
        }
    }
    for (loop_var, source, body) in parse_for_loops(html) {
        if !keys.iter().any(|k| k == &source) {
            continue;
        }
        let (_bare, fields) = dotted_fields_in_body(&body, &loop_var);
        if fields.is_empty() {
            types.insert(source, ContextValueType::StringArray);
        } else {
            types.insert(
                source,
                ContextValueType::ObjectArray {
                    fields: fields
                        .into_iter()
                        .map(|key| ObjectField {
                            label: humanize_placeholder(&key),
                            key,
                        })
                        .collect(),
                },
            );
        }
    }

    keys.into_iter()
        .map(|key| {
            let value_type = types
                .remove(&key)
                .unwrap_or(ContextValueType::String { multiline: false });
            TemplateVarSpec {
                label: humanize_placeholder(&key),
                value_type,
                key,
            }
        })
        .collect()
}

/// Build labeled inputs and the JSON object they submit from template placeholders.
pub fn posted_template_context(
    html: &str,
    subject: &str,
    mut value_for_input: impl FnMut(&str) -> String,
) -> (Vec<TemplateContextField>, Result<JsonValue, String>) {
    let specs = template_context_specs(html, subject);
    let fields: Vec<TemplateContextField> = specs
        .iter()
        .map(|spec| {
            let input_name = context_input_name(&spec.key);
            TemplateContextField {
                label: spec.label.clone(),
                multiline: spec.is_multiline(),
                hint: spec.hint(),
                value: value_for_input(&input_name),
                input_name,
                key: spec.key.clone(),
            }
        })
        .collect();
    let mut map = Map::new();
    for (spec, field) in specs.iter().zip(fields.iter()) {
        match parse_posted_field(spec, &field.value) {
            Ok(value) => {
                map.insert(spec.key.clone(), value);
            }
            Err(e) => return (fields, Err(e)),
        }
    }
    (fields, Ok(JsonValue::Object(map)))
}

fn parse_posted_field(spec: &TemplateVarSpec, raw: &str) -> Result<JsonValue, String> {
    match &spec.value_type {
        ContextValueType::String { .. } => Ok(json!(raw)),
        ContextValueType::StringArray | ContextValueType::ObjectArray { .. } => {
            let raw = raw.trim();
            let value = if raw.is_empty() {
                json!([])
            } else {
                serde_json::from_str(raw)
                    .map_err(|e| format!("context.{}: invalid JSON ({e})", spec.key))?
            };
            validate_context_value(spec, &value)?;
            Ok(value)
        }
    }
}

/// JSON Schema-like description of `context` for `send_email_to_subscribers`.
pub fn email_template_context_schema_json(html: &str) -> JsonValue {
    let specs = template_context_specs(html, "");
    let mut properties = Map::new();
    let mut required = Vec::new();
    let mut sig_parts = Vec::new();
    for spec in &specs {
        properties.insert(spec.key.clone(), spec.schema_property());
        required.push(json!(spec.key));
        sig_parts.push(spec.signature_part());
    }
    let signature = if sig_parts.is_empty() {
        "#{}".to_string()
    } else {
        format!("#{{ {} }}", sig_parts.join(", "))
    };
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": required,
        "properties": properties,
        "signature": signature,
    })
}

/// `context` must contain exactly the template argument schema.
pub fn validate_template_context(
    context: &Map<String, JsonValue>,
    specs: &[TemplateVarSpec],
) -> Result<(), String> {
    let keys: Vec<String> = specs.iter().map(|s| s.key.clone()).collect();
    let mut unknown: Vec<&str> = context
        .keys()
        .filter(|k| !keys.iter().any(|p| p == *k))
        .map(|k| k.as_str())
        .collect();
    unknown.sort_unstable();
    if !unknown.is_empty() {
        return Err(format!(
            "context has unknown field(s) {}. Expected {}. Call email_template_context_schema(()) for the schema.",
            quoted_keys(&unknown),
            expected_context_keys(&keys),
        ));
    }
    let missing: Vec<&str> = keys
        .iter()
        .filter(|k| !context.contains_key(*k))
        .map(|k| k.as_str())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "context missing field(s) {}. Call email_template_context_schema(()) for the schema.",
            quoted_keys(&missing),
        ));
    }
    for spec in specs {
        let Some(value) = context.get(&spec.key) else {
            unreachable!("checked missing keys");
        };
        validate_context_value(spec, value)?;
    }
    Ok(())
}

fn validate_context_value(spec: &TemplateVarSpec, value: &JsonValue) -> Result<(), String> {
    match &spec.value_type {
        ContextValueType::String { .. } => {
            if value.is_string() {
                Ok(())
            } else {
                Err(format!(
                    "context.{} must be a string, got {value}",
                    spec.key
                ))
            }
        }
        ContextValueType::StringArray => match value {
            JsonValue::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    if !item.is_string() {
                        return Err(format!(
                            "context.{}[{i}] must be a string, got {item}",
                            spec.key
                        ));
                    }
                }
                Ok(())
            }
            other => Err(format!(
                "context.{} must be an array of strings, got {other}",
                spec.key
            )),
        },
        ContextValueType::ObjectArray { fields } => match value {
            JsonValue::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    validate_object_item(&spec.key, i, item, fields)?;
                }
                Ok(())
            }
            other => Err(format!(
                "context.{} must be an array of objects, got {other}",
                spec.key
            )),
        },
    }
}

fn validate_object_item(
    key: &str,
    i: usize,
    item: &JsonValue,
    fields: &[ObjectField],
) -> Result<(), String> {
    let JsonValue::Object(map) = item else {
        return Err(format!("context.{key}[{i}] must be an object, got {item}"));
    };
    let allowed: Vec<&str> = fields.iter().map(|f| f.key.as_str()).collect();
    let mut unknown: Vec<&str> = map
        .keys()
        .filter(|k| !allowed.iter().any(|a| a == k))
        .map(|k| k.as_str())
        .collect();
    unknown.sort_unstable();
    if !unknown.is_empty() {
        return Err(format!(
            "context.{key}[{i}] has unknown field(s) {}",
            quoted_keys(&unknown)
        ));
    }
    for field in fields {
        match map.get(&field.key) {
            Some(JsonValue::String(_)) => {}
            Some(other) => {
                return Err(format!(
                    "context.{key}[{i}].{} must be a string, got {other}",
                    field.key
                ));
            }
            None => {
                return Err(format!("context.{key}[{i}] missing field `{}`", field.key));
            }
        }
    }
    Ok(())
}

fn quoted_keys(keys: &[&str]) -> String {
    keys.iter()
        .map(|k| format!("`{k}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn expected_context_keys(placeholders: &[String]) -> String {
    if placeholders.is_empty() {
        "no fields (omit context or pass #{})".into()
    } else {
        quoted_keys(&placeholders.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }
}

/// Merge subscriber identity into a template context object.
pub fn subscriber_template_context(base: &JsonValue, subscriber_email: &str) -> JsonValue {
    let mut map = match base {
        JsonValue::Object(m) => m.clone(),
        JsonValue::Null => Map::new(),
        other => {
            let mut m = Map::new();
            m.insert("value".into(), other.clone());
            m
        }
    };
    map.insert("email".into(), json!(subscriber_email));
    map.insert("subscriber_email".into(), json!(subscriber_email));
    JsonValue::Object(map)
}

/// HTML escape for email: `& < > " '` but not `/`, so URLs in attributes stay intact.
fn write_email_html_escaped(out: &mut Output, s: &str) -> Result<(), MiniError> {
    let mut last = 0;
    for (i, ch) in s.char_indices() {
        let repl = match ch {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&#x27;",
            _ => continue,
        };
        if last < i {
            out.write_str(&s[last..i]).map_err(MiniError::from)?;
        }
        out.write_str(repl).map_err(MiniError::from)?;
        last = i + ch.len_utf8();
    }
    if last < s.len() {
        out.write_str(&s[last..]).map_err(MiniError::from)?;
    }
    Ok(())
}

fn email_html_formatter(
    out: &mut Output,
    state: &State,
    value: &MiniValue,
) -> Result<(), MiniError> {
    if !matches!(state.auto_escape(), AutoEscape::Html) {
        return escape_formatter(out, state, value);
    }
    if value.is_safe() {
        return write!(out, "{value}").map_err(MiniError::from);
    }
    if let Some(s) = value.as_str() {
        return write_email_html_escaped(out, s);
    }
    write_email_html_escaped(out, &value.to_string())
}

fn render_template(
    src: &str,
    context: &JsonValue,
    auto_escape: AutoEscape,
) -> Result<String, EmailSendError> {
    let mut env = Environment::new();
    env.set_auto_escape_callback(move |_| auto_escape);
    if matches!(auto_escape, AutoEscape::Html) {
        env.set_formatter(email_html_formatter);
    }
    let tmpl = env
        .template_from_str(src)
        .map_err(|e| EmailSendError::Template(e.to_string()))?;
    tmpl.render(MiniValue::from_serialize(context))
        .map_err(|e| EmailSendError::Template(e.to_string()))
}

/// Render HTML email with HTML auto-escape (does not double-escape template entities).
pub fn render_email_html(template: &str, context: &JsonValue) -> Result<String, EmailSendError> {
    render_template(template, context, AutoEscape::Html)
}

/// Render a subject line without HTML escaping.
pub fn render_email_subject(template: &str, context: &JsonValue) -> Result<String, EmailSendError> {
    render_template(template, context, AutoEscape::None)
}

/// Send rendered HTML to each recipient using publisher SMTP preferences.
pub async fn send_subscriber_emails(
    prefs: &PublisherPreferences,
    recipients: &[String],
    subject: &str,
    html: &str,
    context: &JsonValue,
    attachments: Vec<PreparedAttachment>,
) -> Result<SendReport, EmailSendError> {
    let host = prefs.smtp_host.trim();
    if host.is_empty() {
        return Err(EmailSendError::Send(
            "SMTP host is not configured in Publisher preferences".into(),
        ));
    }
    let from_addr = prefs.smtp_from.trim();
    if from_addr.is_empty() {
        return Err(EmailSendError::Send(
            "SMTP from address is not configured in Publisher preferences".into(),
        ));
    }
    let from: Mailbox = from_addr
        .parse()
        .map_err(|e| EmailSendError::From(format!("{e}")))?;

    let context = match context {
        JsonValue::Null => json!({}),
        other => other.clone(),
    };
    let mut messages = Vec::with_capacity(recipients.len());
    for email in recipients {
        let ctx = subscriber_template_context(&context, email);
        let subject = render_email_subject(subject, &ctx)?;
        let html = render_email_html(html, &ctx)?;
        messages.push((email.clone(), subject, html));
    }

    let port: u16 = prefs.smtp_port.trim().parse().unwrap_or(587);
    let username = prefs.smtp_username.clone();
    let password = prefs.smtp_password.clone();
    let host = host.to_string();

    tokio::task::spawn_blocking(move || {
        let tls = TlsParameters::builder(host.clone())
            .build()
            .map_err(|e| EmailSendError::Send(e.to_string()))?;
        let tls_mode = if port == 465 {
            Tls::Wrapper(tls)
        } else {
            Tls::Required(tls)
        };
        let mut builder = SmtpTransport::relay(&host)
            .map_err(|e| EmailSendError::Send(e.to_string()))?
            .port(port)
            .tls(tls_mode);
        if !username.is_empty() {
            builder = builder.credentials(Credentials::new(username, password));
        }
        let mailer = builder.build();

        let mut report = SendReport::default();
        for (to, subject, html) in messages {
            let to_mailbox: Mailbox = match to.parse() {
                Ok(m) => m,
                Err(e) => {
                    report.failures.push((to, format!("invalid address: {e}")));
                    continue;
                }
            };
            let body = mixed_body(&html, &attachments);
            let email = match Message::builder()
                .from(from.clone())
                .to(to_mailbox)
                .subject(&subject)
                .multipart(body)
            {
                Ok(m) => m,
                Err(e) => {
                    report.failures.push((to, e.to_string()));
                    continue;
                }
            };
            match mailer.send(&email) {
                Ok(_) => report.sent += 1,
                Err(e) => report.failures.push((to, e.to_string())),
            }
        }
        Ok(report)
    })
    .await
    .map_err(|e| EmailSendError::Send(e.to_string()))?
}

/// Send a transactional notification when an edit subscription link has been opened.
pub const DEFAULT_EDIT_LINK_EMAIL_SUBJECT: &str =
    "Change your Resummarized subscription preferences";

pub const DEFAULT_EDIT_LINK_EMAIL_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.5; color: #111; background-color: #fafafa; padding: 24px; }
    .card { max-width: 560px; margin: 0 auto; background: #fff; border: 1px solid #e5e5e5; border-radius: 8px; padding: 32px; }
    h2 { margin-top: 0; color: #111; font-size: 20px; }
    p { color: #444; font-size: 15px; margin: 16px 0; }
    .btn { display: inline-block; background-color: #111; color: #fff !important; text-decoration: none; padding: 12px 24px; border-radius: 6px; font-weight: 500; font-size: 14px; margin: 16px 0; }
    .footer { font-size: 12px; color: #888; margin-top: 24px; border-top: 1px solid #eee; padding-top: 16px; word-break: break-all; }
  </style>
</head>
<body>
  <div class="card">
    <h2>Manage Your Subscription</h2>
    <p>You can manage and update your subscription preferences for Resummarized using your secure link below.</p>
    <p>Customize your delivery cadence and filter stock market updates by exchange, company, or event type:</p>
    <p><a href="{{ edit_link }}" class="btn">Update Subscription Preferences</a></p>
    <p class="footer">Or copy and paste this URL into your browser:<br>{{ edit_link }}</p>
    <p class="footer">If you did not request this link, you can safely ignore this email.</p>
  </div>
</body>
</html>"#;

pub const DEFAULT_EDIT_OPENED_EMAIL_SUBJECT: &str =
    "Your Resummarized subscription preferences link";

pub const DEFAULT_EDIT_OPENED_EMAIL_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.5; color: #111; background-color: #fafafa; padding: 24px; }
    .card { max-width: 560px; margin: 0 auto; background: #fff; border: 1px solid #e5e5e5; border-radius: 8px; padding: 32px; }
    h2 { margin-top: 0; color: #111; font-size: 20px; }
    p { color: #444; font-size: 15px; margin: 16px 0; }
    .btn { display: inline-block; background-color: #111; color: #fff !important; text-decoration: none; padding: 12px 24px; border-radius: 6px; font-weight: 500; font-size: 14px; margin: 16px 0; }
    .footer { font-size: 12px; color: #888; margin-top: 24px; border-top: 1px solid #eee; padding-top: 16px; word-break: break-all; }
  </style>
</head>
<body>
  <div class="card">
    <h2>Subscription Preferences Accessed</h2>
    <p>We noticed that your subscription edit page was recently opened. For your security, your one-time link has been regenerated.</p>
    <p>If you still need to update your preferences, or if you closed your browser tab, you can use your new link below:</p>
    <p><a href="{{ edit_link }}" class="btn">Update Subscription Preferences</a></p>
    <p class="footer">Or copy and paste this URL into your browser:<br>{{ edit_link }}</p>
    <p class="footer">If you did not make this request, you can safely ignore this email.</p>
  </div>
</body>
</html>"#;

pub const DEFAULT_EDIT_UPDATED_EMAIL_SUBJECT: &str =
    "Your Resummarized subscription preferences were updated";

pub const DEFAULT_EDIT_UPDATED_EMAIL_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.5; color: #111; background-color: #fafafa; padding: 24px; }
    .card { max-width: 560px; margin: 0 auto; background: #fff; border: 1px solid #e5e5e5; border-radius: 8px; padding: 32px; }
    h2 { margin-top: 0; color: #111; font-size: 20px; }
    p { color: #444; font-size: 15px; margin: 16px 0; }
    .btn { display: inline-block; background-color: #111; color: #fff !important; text-decoration: none; padding: 12px 24px; border-radius: 6px; font-weight: 500; font-size: 14px; margin: 16px 0; }
    .footer { font-size: 12px; color: #888; margin-top: 24px; border-top: 1px solid #eee; padding-top: 16px; word-break: break-all; }
  </style>
</head>
<body>
  <div class="card">
    <h2>Subscription Preferences Updated</h2>
    <p>Your subscription preferences for Resummarized have been updated successfully.</p>
    <p>If you need to make changes in the future, you can access your edit page anytime using your new secure link:</p>
    <p><a href="{{ edit_link }}" class="btn">Manage Preferences</a></p>
    <p class="footer">Or copy and paste this URL into your browser:<br>{{ edit_link }}</p>
    <p class="footer">If you did not perform this change, please contact support.</p>
  </div>
</body>
</html>"#;

/// Send an email to a subscriber inviting them to change/manage their subscription preferences.
pub async fn send_change_subscription_email(
    prefs: &PublisherPreferences,
    recipient: &str,
    edit_link: &str,
) -> Result<(), EmailSendError> {
    let subject_tmpl = if prefs.edit_link_email_subject.trim().is_empty() {
        DEFAULT_EDIT_LINK_EMAIL_SUBJECT
    } else {
        prefs.edit_link_email_subject.as_str()
    };
    let html_tmpl = if prefs.edit_link_email_template.trim().is_empty() {
        DEFAULT_EDIT_LINK_EMAIL_TEMPLATE
    } else {
        prefs.edit_link_email_template.as_str()
    };
    let ctx = serde_json::json!({
        "edit_link": edit_link,
        "email": recipient,
    });
    let subject = render_email_subject(subject_tmpl, &ctx)?;
    let html = render_email_html(html_tmpl, &ctx)?;
    send_subscriber_emails(
        prefs,
        &[recipient.to_string()],
        &subject,
        &html,
        &serde_json::json!({}),
        Vec::new(),
    )
    .await
    .map(|_| ())
}

/// Send a transactional notification when the subscription edit page is opened.
pub async fn send_edit_link_opened_email(
    prefs: &PublisherPreferences,
    recipient: &str,
    new_edit_link: &str,
) -> Result<(), EmailSendError> {
    let subject_tmpl = if prefs.edit_opened_email_subject.trim().is_empty() {
        DEFAULT_EDIT_OPENED_EMAIL_SUBJECT
    } else {
        prefs.edit_opened_email_subject.as_str()
    };
    let html_tmpl = if prefs.edit_opened_email_template.trim().is_empty() {
        DEFAULT_EDIT_OPENED_EMAIL_TEMPLATE
    } else {
        prefs.edit_opened_email_template.as_str()
    };
    let ctx = serde_json::json!({
        "edit_link": new_edit_link,
        "email": recipient,
    });
    let subject = render_email_subject(subject_tmpl, &ctx)?;
    let html = render_email_html(html_tmpl, &ctx)?;
    send_subscriber_emails(
        prefs,
        &[recipient.to_string()],
        &subject,
        &html,
        &serde_json::json!({}),
        Vec::new(),
    )
    .await
    .map(|_| ())
}

/// Send a transactional notification when subscription preferences have been updated.
pub async fn send_subscription_updated_email(
    prefs: &PublisherPreferences,
    recipient: &str,
    new_edit_link: &str,
) -> Result<(), EmailSendError> {
    let subject_tmpl = if prefs.edit_updated_email_subject.trim().is_empty() {
        DEFAULT_EDIT_UPDATED_EMAIL_SUBJECT
    } else {
        prefs.edit_updated_email_subject.as_str()
    };
    let html_tmpl = if prefs.edit_updated_email_template.trim().is_empty() {
        DEFAULT_EDIT_UPDATED_EMAIL_TEMPLATE
    } else {
        prefs.edit_updated_email_template.as_str()
    };
    let ctx = serde_json::json!({
        "edit_link": new_edit_link,
        "email": recipient,
    });
    let subject = render_email_subject(subject_tmpl, &ctx)?;
    let html = render_email_html(html_tmpl, &ctx)?;
    send_subscriber_emails(
        prefs,
        &[recipient.to_string()],
        &subject,
        &html,
        &serde_json::json!({}),
        Vec::new(),
    )
    .await
    .map(|_| ())
}

/// Default daily-market HTML email (lists for highlights and upcoming actions).
pub const DEFAULT_EMAIL_HTML_TEMPLATE: &str = include_str!("email_template.html");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_for_common_extensions() {
        assert_eq!(mime_for_name("report.pdf"), "application/pdf");
        assert_eq!(mime_for_name("photo.JPEG"), "image/jpeg");
        assert_eq!(mime_for_name("notes"), "application/octet-stream");
    }

    const MARKET_STATUS_TEMPLATE: &str = DEFAULT_EMAIL_HTML_TEMPLATE;

    fn market_status_context() -> JsonValue {
        json!({
            "report_date": "11 Sep 2026",
            "sensex_close": "81,234.56",
            "sensex_change": "+245.10",
            "sensex_pct": "+0.30%",
            "highlights": [
                {
                    "name": "Ratnaveer Precision Engineering Ltd (BSE: 543978 / NSE: RATNAVEER)",
                    "disclosure": "Rights issue allotment approved for 12,499,669 equity shares at ₹264 per share (mobilizing ₹330 Crores)."
                },
                {
                    "name": "Texmaco Rail & Engineering Ltd (BSE: 533326 / NSE: TEXRAIL)",
                    "disclosure": "Secured commercial purchase contract worth ₹27.82 Crores (inclusive of GST) from Hindalco Industries Limited."
                },
                {
                    "name": "Tata Steel & Co",
                    "disclosure": "Announced capacity expansion in Odisha."
                }
            ],
            "next_ex_date": "11 September 2026",
            "upcoming_actions": [
                {
                    "company": "Finolex Industries Limited (NSE: FINPIPE / BSE: 500940)",
                    "action": "Final Dividend ₹2.00 + Special Dividend ₹0.75",
                    "record_date": "11-Sep-2026"
                },
                {
                    "company": "Steel City Securities Limited (NSE: STEELCITY / BSE: 542910)",
                    "action": "Interim Dividend ₹1.00 per share",
                    "record_date": "11-Sep-2026"
                },
                {
                    "company": "Sri Lotus Developers and Realty (NSE: SRILOTUS)",
                    "action": "Interim Dividend ₹0.30 per share",
                    "record_date": "11-Sep-2026"
                }
            ],
            "pdf_download_url": "https://example.com/report.pdf",
            "service_name": "Resummarized",
            "unsubscribe_url": "https://example.com/unsubscribe",
        })
    }

    #[test]
    fn market_status_html_template_renders() {
        let ctx = subscriber_template_context(&market_status_context(), "reader@example.com");
        let html = render_email_html(MARKET_STATUS_TEMPLATE, &ctx).expect("render");

        assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
        assert!(
            html.contains("<!-- Header -->"),
            "HTML comments must survive"
        );
        assert!(
            html.contains("body { margin: 0; padding: 0;"),
            "CSS braces must survive: {html}"
        );
        assert!(
            html.contains("linear-gradient(135deg, #0f172a 0%, #1e293b 100%)"),
            "CSS gradients/percentages must survive"
        );
        assert!(
            html.contains("color: #ffffff !important;"),
            "CSS !important must survive"
        );
        assert!(
            html.contains("BSE &amp; NSE Daily Market Status Report"),
            "literal &amp; must not double-escape"
        );
        assert!(
            !html.contains("BSE &amp;amp; NSE"),
            "literal &amp; was double-escaped: {html}"
        );
        assert!(html.contains("S&amp;P BSE SENSEX"), "{html}");
        assert!(html.contains("Session: 11 Sep 2026 | Coverage:"), "{html}");
        assert!(html.contains(">81,234.56<"), "{html}");
        assert!(html.contains("+245.10 (+0.30%)"), "{html}");
        assert!(!html.contains("FEED INTEGRITY"), "{html}");
        assert!(!html.contains("DISCLOSURES"), "{html}");
        assert!(!html.contains("Infrastructure Health"), "{html}");
        assert!(html.contains("RATNAVEER"), "{html}");
        assert!(html.contains("mobilizing ₹330 Crores"), "{html}");
        assert!(
            html.contains("Tata Steel &amp; Co:"),
            "variable ampersands must be escaped: {html}"
        );
        assert!(html.contains("SRILOTUS"), "{html}");
        assert!(html.contains("Ex-Date: 11 September 2026"), "{html}");
        assert!(html.contains("11-Sep-2026"), "{html}");
        assert!(
            html.contains("href=\"https://example.com/report.pdf\""),
            "{html}"
        );
        assert!(html.contains("subscribers of Resummarized."), "{html}");
        assert!(
            html.contains("href=\"https://example.com/unsubscribe\""),
            "{html}"
        );
        assert!(
            !html.contains("{{report_date}}"),
            "placeholder left unrendered"
        );
        assert!(!html.contains("{{unsubscribe_url}}"), "{html}");
    }

    #[test]
    fn html_autoescape_does_not_break_subject() {
        let ctx = json!({ "name": "Foo & Bar" });
        let subject = render_email_subject("Hello {{name}}", &ctx).expect("subject");
        assert_eq!(subject, "Hello Foo & Bar");
        let html = render_email_html("<p>{{name}}</p>", &ctx).expect("html");
        assert_eq!(html, "<p>Foo &amp; Bar</p>");
    }

    #[test]
    fn market_status_placeholders_become_kv_fields() {
        let (fields, ctx) =
            posted_template_context(MARKET_STATUS_TEMPLATE, "Daily {{service_name}}", |_| {
                String::new()
            });
        let ctx = ctx.expect("posted");
        let keys: Vec<&str> = fields.iter().map(|f| f.key.as_str()).collect();
        assert!(keys.contains(&"report_date"), "{keys:?}");
        assert!(keys.contains(&"pdf_download_url"), "{keys:?}");
        assert!(keys.contains(&"highlights"), "{keys:?}");
        assert!(keys.contains(&"upcoming_actions"), "{keys:?}");
        assert!(!keys.contains(&"bse_feeds"), "{keys:?}");
        assert!(!keys.contains(&"feed_integrity_pct"), "{keys:?}");
        assert!(!keys.contains(&"total_filings_count"), "{keys:?}");
        assert!(keys.contains(&"service_name"), "{keys:?}");
        assert!(!keys.contains(&"email"), "{keys:?}");
        assert!(!keys.contains(&"subscriber_email"), "{keys:?}");
        assert!(!keys.contains(&"company_1_name"), "{keys:?}");
        assert_eq!(
            fields
                .iter()
                .find(|f| f.key == "report_date")
                .unwrap()
                .label,
            "Report Date"
        );
        assert!(
            fields
                .iter()
                .find(|f| f.key == "highlights")
                .unwrap()
                .multiline
        );
        assert!(
            !fields
                .iter()
                .find(|f| f.key == "report_date")
                .unwrap()
                .multiline
        );
        assert_eq!(ctx["report_date"], json!(""));
        assert_eq!(ctx["highlights"], json!([]));
        assert_eq!(ctx["upcoming_actions"], json!([]));
    }

    #[test]
    fn posted_values_fill_template_context() {
        let (fields, _ctx) =
            posted_template_context("<p>{{report_date}} {{email}}</p>", "", |name| {
                if name == "ctx[report_date]" {
                    "11 Sep 2026".into()
                } else {
                    String::new()
                }
            });
        let _ctx = _ctx.expect("posted");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "report_date");
        assert_eq!(fields[0].value, "11 Sep 2026");
        assert_eq!(fields[0].input_name, "ctx[report_date]");
    }

    #[test]
    fn css_and_entities_are_not_placeholders() {
        let keys = template_placeholders(MARKET_STATUS_TEMPLATE);
        assert!(!keys.iter().any(|k| k.contains("margin")), "{keys:?}");
        assert!(!keys.is_empty(), "{keys:?}");
    }

    #[test]
    fn template_context_schema_lists_placeholders() {
        let schema = email_template_context_schema_json(MARKET_STATUS_TEMPLATE);
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["additionalProperties"], false);
        let required = schema["required"].as_array().expect("required");
        assert!(required.iter().any(|v| v == "report_date"), "{required:?}");
        assert!(
            required.iter().any(|v| v == "pdf_download_url"),
            "{required:?}"
        );
        assert!(!required.iter().any(|v| v == "email"), "{required:?}");
        assert_eq!(schema["properties"]["report_date"]["type"], "string");
        assert_eq!(schema["properties"]["report_date"]["label"], "Report Date");
        assert_eq!(schema["properties"]["highlights"]["type"], "array");
        assert_eq!(
            schema["properties"]["highlights"]["items"]["required"],
            json!(["name", "disclosure"])
        );
        let signature = schema["signature"].as_str().expect("signature");
        assert!(signature.starts_with("#{ "), "{signature}");
        assert!(signature.contains("report_date: string"), "{signature}");
        assert!(!signature.contains("bse_feeds"), "{signature}");
        assert!(
            signature.contains("highlights: [#{ name: string, disclosure: string }]"),
            "{signature}"
        );
    }

    #[test]
    fn template_context_schema_empty_template() {
        let schema = email_template_context_schema_json("<p>Hello</p>");
        assert_eq!(schema["signature"], json!("#{}"));
        assert_eq!(schema["required"], json!([]));
        assert_eq!(schema["properties"], json!({}));
    }

    #[test]
    fn validate_template_context_requires_exact_string_keys() {
        let specs = template_context_specs("{{report_date}}", "");
        assert!(validate_template_context(&Map::new(), &specs).is_err());

        let mut extra = Map::new();
        extra.insert("report_date".into(), json!("x"));
        extra.insert("nope".into(), json!("y"));
        let err = validate_template_context(&extra, &specs).unwrap_err();
        assert!(err.contains("unknown"), "{err}");
        assert!(err.contains("nope"), "{err}");

        let mut number = Map::new();
        number.insert("report_date".into(), json!(1));
        let err = validate_template_context(&number, &specs).unwrap_err();
        assert!(err.contains("string"), "{err}");

        let mut ok = Map::new();
        ok.insert("report_date".into(), json!("11 Sep 2026"));
        assert!(validate_template_context(&ok, &specs).is_ok());

        assert!(validate_template_context(&Map::new(), &[]).is_ok());
    }

    #[test]
    fn validate_template_context_accepts_report_lists() {
        let specs = template_context_specs(MARKET_STATUS_TEMPLATE, "");
        let ctx = market_status_context();
        let JsonValue::Object(map) = ctx else {
            panic!("object");
        };
        validate_template_context(&map, &specs).expect("sample report context matches schema");
    }

    #[test]
    fn subscription_email_templates_render_with_context() {
        let ctx = serde_json::json!({
            "edit_link": "https://example.com/subscribe/edit?email=test%40example.com&one_time_token=123",
            "email": "test@example.com",
        });

        // Test default templates
        let def_subj = render_email_subject(DEFAULT_EDIT_LINK_EMAIL_SUBJECT, &ctx).expect("render subject");
        assert_eq!(def_subj, "Change your Resummarized subscription preferences");

        let def_html = render_email_html(DEFAULT_EDIT_LINK_EMAIL_TEMPLATE, &ctx).expect("render html");
        assert!(def_html.contains("https://example.com/subscribe/edit?email=test%40example.com&amp;one_time_token=123") || def_html.contains("https://example.com/subscribe/edit?email=test%40example.com&one_time_token=123"));
        assert!(def_html.contains("Update Subscription Preferences"));

        // Test custom template from preferences
        let custom_subj = render_email_subject("Custom update for {{ email }}", &ctx).expect("render custom subject");
        assert_eq!(custom_subj, "Custom update for test@example.com");

        let custom_html = render_email_html("<a href=\"{{ edit_link }}\">Click here to manage {{ email }}</a>", &ctx).expect("render custom html");
        assert!(custom_html.contains("https://example.com/subscribe/edit?email=test%40example.com"));
        assert!(custom_html.contains("test@example.com"));
    }
}
