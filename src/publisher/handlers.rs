use axum::{
    body::Bytes,
    extract::{FromRequest, Path, Query, Request},
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use lariv_rs::{
    components::{
        DEFAULT_PAGE_SIZE, ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx, SwapKey,
    },
    html_form::{
        CsrfToken, FormError, HtmlFormBody, UrlencodedFields, csrf::csrf_rejection,
        verify_form_csrf,
    },
    http::Cap,
    plugins::{
        filesystem::{
            entities::{VNodeEntity, filesystem_node::Column as VNodeColumn},
            state::FilesystemState,
        },
        users::middleware::RequireAuth,
    },
    template::RenderAppPane,
    web::{
        Htmx, ModalFormQuery as ModalNameQuery, QueryPage, html_built_page_or_app_layout,
        html_built_page_with_slots, respond_create_modal_done, respond_edit_modal_done,
    },
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};

use super::{
    email::{
        TemplateContextField, load_attachments, posted_template_context, send_subscriber_emails,
    },
    entities::{
        PublisherPreferences,
        subscriber::{self, Entity as SubscriberEntity},
    },
    forms::{PublicSubscribeForm, SendEmailForm, SubscriberForm},
    keys::{
        SubscriberCreateModalKey, SubscriberDeleteModalKey, SubscriberEditModalKey,
        SubscriberSendEmailModalKey, SubscriberTableKey,
    },
    preferences::{empty_preferences, load_preferences, save_preferences},
    routes::{
        PublisherPrefsGetRouteTag, SubscriberDefaultRouteTag, SubscriberDeletePostRouteTag,
        SubscriberDetailRouteTag,
    },
    scope::{
        apply_email_filter, apply_subscriber_sort, email_in_use, email_looks_valid,
        find_subscriber_scoped, is_unique_violation, normalize_subscriber_email, scope_superuser,
    },
    state::PublisherState,
    templates::{
        ConfirmDeletePage, PublisherPreferencesPage, SubscriberCreateModalPage,
        SubscriberDetailPage, SubscriberEditModalPage, SubscriberListPage, SubscriberRow,
        SubscriberSendEmailModalPage,
    },
};

const DELETE_FORM: &str = "publisher.SubscriberDeleteForm";

#[derive(Debug, serde::Deserialize, Default)]
pub struct SubscriberListQuery {
    #[serde(default, rename = "Email", alias = "email")]
    pub email: Option<String>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: QueryPage,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

fn list_url() -> String {
    SubscriberDefaultRouteTag.url()
}

async fn load_subscriber_rows(
    db: &sea_orm::DatabaseConnection,
    q: &SubscriberListQuery,
    auth: &lariv_rs::plugins::users::state::AuthContext,
    page_size: u32,
) -> ObjectList<SubscriberRow> {
    let mut query = SubscriberEntity::find();
    query = apply_email_filter(query, q.email.as_deref());
    query = scope_superuser(query, auth);
    query = apply_subscriber_sort(query, q.sort.as_deref());
    let page = q.page.get();
    let paginator = query.paginate(db, page_size as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows = models
        .into_iter()
        .map(|m| SubscriberRow {
            id: m.id,
            email: m.email,
            subscription_date: auth.format_datetime(m.subscription_date).into_string(),
        })
        .collect();
    ObjectList::from_page(rows, page, page_size, total)
}

pub async fn list(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SubscriberListQuery>,
) -> maud::Markup {
    let subscribers = load_subscriber_rows(&state.db, &q, &ctx, DEFAULT_PAGE_SIZE).await;
    let page = SubscriberListPage {
        subscribers,
        filter_email: q.email.clone().unwrap_or_default(),
        sort: q.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        can_edit: ctx.user.is_superuser,
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<SubscriberTableKey>() {
        return page.render_table();
    }
    if htmx.wants_main_content() {
        return page.render_main().into();
    }
    if htmx.wants_app_layout() {
        return page.render_pane().into();
    }
    html_built_page_with_slots(&page, &chrome, &slot_ctx)
}

pub async fn detail(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(m) = find_subscriber_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&list_url()).into_response();
    };
    let page = SubscriberDetailPage {
        id: m.id,
        email: m.email,
        subscription_date: ctx.format_datetime(m.subscription_date).into_string(),
        can_edit: ctx.user.is_superuser,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> maud::Markup {
    if !ctx.user.is_superuser {
        return maud::html! { div class="alert alert-error" { "Forbidden" } };
    }
    let page = SubscriberCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        email: String::new(),
        subscription_date: ctx.datetime_local_input(Utc::now()).into_string(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

fn create_modal_from_form(
    form: &SubscriberForm,
    q: &ModalNameQuery,
    error: String,
) -> SubscriberCreateModalPage {
    SubscriberCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        email: form.email.clone(),
        subscription_date: form.subscription_date.clone(),
        error,
    }
}

pub async fn create_post(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    HtmlFormBody(form): HtmlFormBody<SubscriberForm>,
) -> Response {
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url()).into_response();
    }
    let email = normalize_subscriber_email(&form.email);
    if email.is_empty() {
        return html_built_page_with_slots(
            &create_modal_from_form(&form, &q, "Email is required".into()),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }
    let Some(subscription_date) = ctx.parse_datetime_local_input(&form.subscription_date) else {
        return html_built_page_with_slots(
            &create_modal_from_form(&form, &q, "Invalid subscription date".into()),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    };
    if email_in_use(&state.db, &email, None).await {
        return html_built_page_with_slots(
            &create_modal_from_form(
                &form,
                &q,
                "A subscriber with this email already exists".into(),
            ),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }
    let now = Utc::now();
    let model = subscriber::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        email: Set(email),
        subscription_date: Set(subscription_date),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done::<SubscriberCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &SubscriberDetailRouteTag::new(saved.id).url(),
        ),
        Err(e) => html_built_page_with_slots(
            &create_modal_from_form(&form, &q, e.to_string()),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response(),
    }
}

fn subscribe_redirect(query: &str) -> Redirect {
    Redirect::to(&format!("/subscribe?{query}"))
}

pub async fn subscribe_post(
    Cap(state): Cap<PublisherState>,
    HtmlFormBody(form): HtmlFormBody<PublicSubscribeForm>,
) -> Redirect {
    let email = normalize_subscriber_email(&form.email);
    if !email_looks_valid(&email) {
        return subscribe_redirect("error=invalid");
    }
    if email_in_use(&state.db, &email, None).await {
        return subscribe_redirect("error=taken");
    }
    let now = Utc::now();
    let model = subscriber::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        email: Set(email),
        subscription_date: Set(now),
    };
    match model.insert(&state.db).await {
        Ok(_) => subscribe_redirect("ok=1"),
        Err(e) if is_unique_violation(&e) => subscribe_redirect("error=taken"),
        Err(e) => {
            tracing::error!(error = %e, "failed to create public subscriber");
            subscribe_redirect("error=save")
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url()).into_response();
    }
    let Some(m) = find_subscriber_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&list_url()).into_response();
    };
    let page = SubscriberEditModalPage {
        id: m.id,
        form_name: q.form_name(),
        email: m.email,
        subscription_date: ctx.datetime_local_input(m.subscription_date).into_string(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

fn edit_modal_from_form(
    id: i64,
    form: &SubscriberForm,
    form_name: String,
    error: String,
) -> SubscriberEditModalPage {
    SubscriberEditModalPage {
        id,
        form_name,
        email: form.email.clone(),
        subscription_date: form.subscription_date.clone(),
        error,
    }
}

pub async fn edit_post(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalNameQuery>,
    HtmlFormBody(form): HtmlFormBody<SubscriberForm>,
) -> Response {
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url()).into_response();
    }
    let Some(existing) = find_subscriber_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&list_url()).into_response();
    };
    let email = normalize_subscriber_email(&form.email);
    if email.is_empty() {
        return html_built_page_with_slots(
            &edit_modal_from_form(id, &form, q.form_name(), "Email is required".into()),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }
    let Some(subscription_date) = ctx.parse_datetime_local_input(&form.subscription_date) else {
        return html_built_page_with_slots(
            &edit_modal_from_form(id, &form, q.form_name(), "Invalid subscription date".into()),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    };
    if email_in_use(&state.db, &email, Some(id)).await {
        return html_built_page_with_slots(
            &edit_modal_from_form(
                id,
                &form,
                q.form_name(),
                "A subscriber with this email already exists".into(),
            ),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }
    let now = Utc::now();
    let mut am: subscriber::ActiveModel = existing.into();
    am.updated_at = Set(Some(now));
    am.email = Set(email);
    am.subscription_date = Set(subscription_date);
    match am.update(&state.db).await {
        Ok(_) => respond_edit_modal_done::<SubscriberEditModalKey>(
            &htmx,
            &SubscriberDetailRouteTag::new(id).url(),
        ),
        Err(e) => html_built_page_with_slots(
            &edit_modal_from_form(id, &form, q.form_name(), e.to_string()),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response(),
    }
}

pub async fn delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
    Path(id): Path<i64>,
) -> maud::Markup {
    let page = ConfirmDeletePage {
        modal_uid: SubscriberDeleteModalKey::ID.to_string(),
        message: "Are you sure you want to delete this subscriber?".into(),
        form_name: q.name.clone().unwrap_or_else(|| DELETE_FORM.into()),
        post_url: SubscriberDeletePostRouteTag::new(id).url(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn delete_post(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let list_url = list_url();
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url).into_response();
    }
    match SubscriberEntity::delete_by_id(id).exec(&state.db).await {
        Ok(_) => htmx.redirect(&list_url),
        Err(e) => {
            tracing::error!(error = %e, id, "failed to delete subscriber");
            let page = ConfirmDeletePage {
                modal_uid: SubscriberDeleteModalKey::ID.to_string(),
                message: "Are you sure you want to delete this subscriber?".into(),
                form_name: DELETE_FORM.into(),
                post_url: SubscriberDeletePostRouteTag::new(id).url(),
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

fn prefs_page(prefs: PublisherPreferences, error: String) -> PublisherPreferencesPage {
    PublisherPreferencesPage {
        html_template: prefs.html_template,
        smtp_host: prefs.smtp_host,
        smtp_port: prefs.smtp_port,
        smtp_username: prefs.smtp_username,
        smtp_password: prefs.smtp_password,
        smtp_from: prefs.smtp_from,
        error,
    }
}

pub async fn preferences_get(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url()).into_response();
    }
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = match load_preferences(&state.db).await {
        Ok(p) => p,
        Err(e) => {
            let page = prefs_page(empty_preferences(), e.to_string());
            return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
        }
    };
    html_built_page_or_app_layout(&prefs_page(prefs, String::new()), &htmx, &chrome, &slot_ctx)
        .into_response()
}

pub async fn preferences_post(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    HtmlFormBody(form): HtmlFormBody<super::forms::PreferencesForm>,
) -> Response {
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url()).into_response();
    }
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let prefs = PublisherPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        html_template: form.html_template,
        smtp_host: form.smtp_host,
        smtp_port: form.smtp_port,
        smtp_username: form.smtp_username,
        smtp_password: form.smtp_password,
        smtp_from: form.smtp_from,
    };
    match save_preferences(&state.db, prefs.clone()).await {
        Ok(_) => htmx.redirect(&PublisherPrefsGetRouteTag.url()),
        Err(e) => {
            let page = prefs_page(prefs, e.to_string());
            html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response()
        }
    }
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct SendEmailQuery {
    #[serde(flatten)]
    pub modal: ModalNameQuery,
    #[serde(default)]
    pub ids: Option<String>,
    #[serde(default)]
    pub all: Option<String>,
}

fn query_is_all(raw: Option<&str>) -> bool {
    matches!(
        raw.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

fn parse_bulk_ids(raw: &str) -> Vec<i64> {
    let mut ids: Vec<i64> = raw
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .filter(|id| *id > 0)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn send_email_modal(
    q: &SendEmailQuery,
    ids: String,
    send_all: bool,
    recipient_count: usize,
    form: &SendEmailForm,
    attachments: Vec<ManyToManyItem>,
    context_fields: Vec<TemplateContextField>,
    error: String,
    can_submit: bool,
) -> SubscriberSendEmailModalPage {
    SubscriberSendEmailModalPage {
        form_name: q.modal.form_name(),
        refresh_table: q.modal.refresh_table(),
        ids,
        send_all,
        recipient_count,
        subject: form.subject.clone(),
        attachments,
        context_fields,
        error,
        can_submit,
    }
}

async fn attachment_items(db: &sea_orm::DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    if ids.is_empty() {
        return Vec::new();
    }
    let nodes = VNodeEntity::find()
        .filter(VNodeColumn::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.iter()
        .filter_map(|id| {
            nodes
                .iter()
                .find(|n| n.id == *id)
                .map(|n| ManyToManyItem::new(n.id.to_string(), n.name.clone()))
        })
        .collect()
}

async fn recipient_emails(
    db: &sea_orm::DatabaseConnection,
    auth: &lariv_rs::plugins::users::state::AuthContext,
    send_all: bool,
    ids: &[i64],
) -> Vec<String> {
    let query = if send_all {
        scope_superuser(SubscriberEntity::find(), auth)
    } else if ids.is_empty() {
        return Vec::new();
    } else {
        scope_superuser(
            SubscriberEntity::find().filter(subscriber::Column::Id.is_in(ids.to_vec())),
            auth,
        )
    };
    query
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.email)
        .collect()
}

pub async fn send_email_get(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<SendEmailQuery>,
) -> maud::Markup {
    if !ctx.user.is_superuser {
        return maud::html! { div class="alert alert-error" { "Forbidden" } };
    }
    let send_all = query_is_all(q.all.as_deref());
    let ids = parse_bulk_ids(q.ids.as_deref().unwrap_or(""));
    let ids_str = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let recipients = recipient_emails(&state.db, &ctx, send_all, &ids).await;
    let (error, can_submit) = if send_all && recipients.is_empty() {
        ("There are no subscribers to email.".into(), false)
    } else if !send_all && ids.is_empty() {
        (
            "Select at least one subscriber, or choose Send to all subscribers.".into(),
            false,
        )
    } else if !send_all && recipients.is_empty() {
        (
            "None of the selected subscribers could be found.".into(),
            false,
        )
    } else {
        (String::new(), true)
    };
    let html_template = load_preferences(&state.db)
        .await
        .map(|p| p.html_template)
        .unwrap_or_default();
    let (context_fields, _) = posted_template_context(&html_template, "", |_| String::new());
    let page = send_email_modal(
        &q,
        ids_str,
        send_all,
        recipients.len(),
        &SendEmailForm {
            subject: String::new(),
            attachments: Vec::new(),
            csrf: CsrfToken::current(),
        },
        Vec::new(),
        context_fields,
        error,
        can_submit,
    );
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub struct SendEmailPosted {
    form: SendEmailForm,
    fields: UrlencodedFields,
}

impl<S> FromRequest<S> for SendEmailPosted
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !content_type.starts_with("application/x-www-form-urlencoded") {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Expected `application/x-www-form-urlencoded` request body".into(),
            ));
        }
        let headers = req.headers().clone();
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
        let fields = UrlencodedFields::parse(&bytes).map_err(send_email_form_rejection)?;
        verify_form_csrf(&headers, &fields).map_err(send_email_form_rejection)?;
        let form: SendEmailForm = fields.deserialize().map_err(send_email_form_rejection)?;
        Ok(Self { form, fields })
    }
}

fn send_email_form_rejection(err: FormError) -> (StatusCode, String) {
    if let Some(rej) = csrf_rejection(&err) {
        return rej;
    }
    (
        StatusCode::BAD_REQUEST,
        format!("Failed to deserialize form body: {err}"),
    )
}

pub async fn send_email_post(
    Cap(state): Cap<PublisherState>,
    Cap(fs): Cap<FilesystemState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<SendEmailQuery>,
    SendEmailPosted { form, fields }: SendEmailPosted,
) -> Response {
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url()).into_response();
    }
    let send_all = query_is_all(q.all.as_deref());
    let ids = parse_bulk_ids(q.ids.as_deref().unwrap_or(""));
    let ids_str = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let attachments = attachment_items(&state.db, &form.attachments).await;
    let recipients = recipient_emails(&state.db, &ctx, send_all, &ids).await;
    let prefs = load_preferences(&state.db).await;
    let html_template = prefs
        .as_ref()
        .map(|p| p.html_template.as_str())
        .unwrap_or("");
    let (context_fields, context) =
        posted_template_context(html_template, form.subject.trim(), |name| {
            fields.get_first(name).unwrap_or("").to_string()
        });

    let redisplay = |error: String, can_submit: bool| {
        send_email_modal(
            &q,
            ids_str.clone(),
            send_all,
            recipients.len(),
            &form,
            attachments.clone(),
            context_fields.clone(),
            error,
            can_submit,
        )
    };
    let context = match context {
        Ok(c) => c,
        Err(e) => {
            return html_built_page_with_slots(
                &redisplay(e, true),
                &chrome,
                &SlotCtx::from_auth(&ctx),
            )
            .into_response();
        }
    };

    if form.subject.trim().is_empty() {
        return html_built_page_with_slots(
            &redisplay("Subject is required".into(), true),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }
    if recipients.is_empty() {
        return html_built_page_with_slots(
            &redisplay("There are no subscribers to email.".into(), false),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }

    let prefs = match prefs {
        Ok(p) => p,
        Err(e) => {
            return html_built_page_with_slots(
                &redisplay(e.to_string(), true),
                &chrome,
                &SlotCtx::from_auth(&ctx),
            )
            .into_response();
        }
    };
    if prefs.html_template.trim().is_empty() {
        return html_built_page_with_slots(
            &redisplay(
                "Set an HTML template in Publisher preferences before sending.".into(),
                true,
            ),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }

    let prepared = match load_attachments(&fs.db, fs.store.as_ref(), &form.attachments).await {
        Ok(a) => a,
        Err(e) => {
            return html_built_page_with_slots(
                &redisplay(e.to_string(), true),
                &chrome,
                &SlotCtx::from_auth(&ctx),
            )
            .into_response();
        }
    };

    match send_subscriber_emails(
        &prefs,
        &recipients,
        form.subject.trim(),
        &prefs.html_template,
        &context,
        prepared,
    )
    .await
    {
        Ok(report) if report.failures.is_empty() => respond_create_modal_done::<
            SubscriberSendEmailModalKey,
        >(
            &htmx, &q.modal.refresh_table(), &list_url()
        ),
        Ok(report) => html_built_page_with_slots(
            &redisplay(report.summary(), true),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response(),
        Err(e) => html_built_page_with_slots(
            &redisplay(e.to_string(), true),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response(),
    }
}
