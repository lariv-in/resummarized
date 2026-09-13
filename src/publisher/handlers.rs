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
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, prelude::Json,
};

use uuid::Uuid;

use crate::stock_markets::StockMarketRegistry;

use super::{
    catalog,
    email::{
        TemplateContextField, load_attachments, posted_template_context, send_change_subscription_email,
        send_edit_link_opened_email, send_subscriber_emails, send_subscription_updated_email,
    },
    entities::{
        PublisherPreferences,
        subscriber::{
            self, Entity as SubscriberEntity, NewsletterInterval, format_filter_display,
            json_to_string_list, normalize_filter_list, normalize_slug_list, string_list_to_json,
        },
    },
    forms::{
        PublicCancelSubscribeForm, PublicEditSubscribeForm, PublicSubscribeForm, SendEmailForm,
        SubscriberForm,
    },
    keys::{
        SubscriberBulkDeleteModalKey, SubscriberBulkSendEditLinkModalKey, SubscriberCreateModalKey,
        SubscriberDeleteModalKey, SubscriberEditModalKey, SubscriberSendEditLinkModalKey,
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
        ConfirmBulkDeletePage, ConfirmDeletePage, PublisherPreferencesPage,
        SubscriberBulkSendEditLinkModalPage, SubscriberCreateModalPage, SubscriberDetailPage,
        SubscriberEditModalPage, SubscriberListPage, SubscriberRow, SubscriberSendEditLinkModalPage,
        SubscriberSendEmailModalPage,
    },
};

const DELETE_FORM: &str = "publisher.SubscriberDeleteForm";
const BULK_DELETE_FORM: &str = "publisher.SubscriberBulkDeleteForm";

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
    let total = match paginator.num_items().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "load_subscriber_rows: failed to count subscribers");
            0
        }
    };
    let models = match paginator.fetch_page((page as u64).saturating_sub(1)).await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(error = %e, "load_subscriber_rows: failed to fetch subscribers page");
            Vec::new()
        }
    };
    let rows = models
        .into_iter()
        .map(|m| SubscriberRow {
            id: m.id,
            email: m.email,
            subscription_date: auth.format_datetime(m.subscription_date).into_string(),
            newsletter_interval: m.newsletter_interval.label().to_string(),
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
    Cap(markets): Cap<StockMarketRegistry>,
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
        filter_exchanges: catalog::format_exchange_display(&m.filter_exchanges, &markets),
        filter_entities: format_filter_display(&m.filter_entities),
        filter_event_types: format_filter_display(&m.filter_event_types),
        newsletter_interval: m.newsletter_interval.label().to_string(),
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
        filter_exchanges: Vec::new(),
        filter_entities: Vec::new(),
        filter_event_types: Vec::new(),
        newsletter_interval: NewsletterInterval::Weekly.as_str().to_string(),
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
        filter_exchanges: form.filter_exchanges.clone(),
        filter_entities: form.filter_entities.clone(),
        filter_event_types: form.filter_event_types.clone(),
        newsletter_interval: form.newsletter_interval.clone(),
        error,
    }
}

fn parsed_subscriber_filters(
    exchanges: Vec<String>,
    entities: Vec<String>,
    event_types: Vec<String>,
    interval: &str,
    markets: &StockMarketRegistry,
) -> Result<(Option<Json>, Option<Json>, Option<Json>, NewsletterInterval), String> {
    let Some(interval) = NewsletterInterval::parse(interval) else {
        return Err("Invalid newsletter interval".into());
    };
    let exchanges = normalize_slug_list(exchanges).unwrap_or_default();
    if let Some(unknown) = exchanges.iter().find(|key| !markets.contains(key)) {
        return Err(format!("Unknown exchange: {unknown}"));
    }
    Ok((
        string_list_to_json(if exchanges.is_empty() {
            None
        } else {
            Some(exchanges)
        }),
        string_list_to_json(normalize_filter_list(entities)),
        string_list_to_json(normalize_slug_list(event_types)),
        interval,
    ))
}

pub async fn create_post(
    Cap(state): Cap<PublisherState>,
    Cap(markets): Cap<StockMarketRegistry>,
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
    let (filter_exchanges, filter_entities, filter_event_types, newsletter_interval) =
        match parsed_subscriber_filters(
            form.filter_exchanges.clone(),
            form.filter_entities.clone(),
            form.filter_event_types.clone(),
            &form.newsletter_interval,
            &markets,
        ) {
            Ok(parsed) => parsed,
            Err(msg) => {
                return html_built_page_with_slots(
                    &create_modal_from_form(&form, &q, msg),
                    &chrome,
                    &SlotCtx::from_auth(&ctx),
                )
                .into_response();
            }
        };
    let now = Utc::now();
    let model = subscriber::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        email: Set(email),
        subscription_date: Set(subscription_date),
        filter_exchanges: Set(filter_exchanges),
        filter_entities: Set(filter_entities),
        filter_event_types: Set(filter_event_types),
        newsletter_interval: Set(newsletter_interval),
        one_time_token: Set(Some(Uuid::new_v4())),
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

pub async fn subscribe_exchanges() -> axum::Json<Vec<catalog::ExchangeOption>> {
    axum::Json(catalog::exchange_options())
}

#[derive(Clone, Debug, serde::Deserialize, Default)]
pub struct SubscribeEventTypesQuery {
    pub exchange: Option<String>,
    pub exchanges: Option<String>,
}

pub async fn subscribe_event_types(
    axum::extract::Query(query): axum::extract::Query<SubscribeEventTypesQuery>,
) -> axum::Json<Vec<catalog::EventTypeOption>> {
    let mut requested: Vec<String> = Vec::new();
    if let Some(ref e) = query.exchange {
        for s in e.split(',') {
            let s = s.trim();
            if !s.is_empty() {
                requested.push(s.to_string());
            }
        }
    }
    if let Some(ref e) = query.exchanges {
        for s in e.split(',') {
            let s = s.trim();
            if !s.is_empty() {
                requested.push(s.to_string());
            }
        }
    }
    if requested.is_empty() {
        axum::Json(catalog::event_type_options())
    } else {
        axum::Json(catalog::event_type_options_for_exchanges(&requested))
    }
}

pub async fn subscribe_post(
    Cap(state): Cap<PublisherState>,
    Cap(markets): Cap<StockMarketRegistry>,
    HtmlFormBody(form): HtmlFormBody<PublicSubscribeForm>,
) -> Redirect {
    let email = normalize_subscriber_email(&form.email);
    if !email_looks_valid(&email) {
        return subscribe_redirect("error=invalid");
    }
    if email_in_use(&state.db, &email, None).await {
        return subscribe_redirect("error=taken");
    }
    let newsletter_interval = if form.newsletter_interval.trim().is_empty() {
        NewsletterInterval::Weekly
    } else {
        match NewsletterInterval::parse(&form.newsletter_interval) {
            Some(interval) => interval,
            None => return subscribe_redirect("error=invalid"),
        }
    };
    let now = Utc::now();
    let filter_exchanges = string_list_to_json({
        let exchanges =
            markets.retain_known(normalize_slug_list(form.filter_exchanges).unwrap_or_default());
        if exchanges.is_empty() {
            None
        } else {
            Some(exchanges)
        }
    });
    let model = subscriber::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        email: Set(email),
        subscription_date: Set(now),
        filter_exchanges: Set(filter_exchanges),
        filter_entities: Set(string_list_to_json(normalize_filter_list(
            form.filter_entities,
        ))),
        filter_event_types: Set(string_list_to_json(normalize_slug_list(
            form.filter_event_types,
        ))),
        newsletter_interval: Set(newsletter_interval),
        one_time_token: Set(Some(Uuid::new_v4())),
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

const SUBSCRIBE_EDIT_HTML: &str = include_str!("../../assets/subscribe_edit.html");
const THEME_CSS_STR: &str = include_str!("../../assets/theme/resummarized.css");

#[derive(Clone, Debug, serde::Deserialize, Default)]
pub struct SubscribeEditQuery {
    pub email: Option<String>,
    pub one_time_token: Option<String>,
}

fn no_cache_html_response(status: StatusCode, html: String) -> Response {
    let mut res = Response::new(axum::body::Body::from(html));
    *res.status_mut() = status;
    let headers = res.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    headers.insert(header::PRAGMA, header::HeaderValue::from_static("no-cache"));
    headers.insert(header::EXPIRES, header::HeaderValue::from_static("0"));
    res
}

fn render_subscribe_error_page(
    db: &sea_orm::DatabaseConnection,
    title: &str,
    heading: &str,
    message: &str,
    status: StatusCode,
) -> Response {
    let mut env = minijinja::Environment::new();
    lariv_rs::plugins::website::template_funcs::register_funcs(
        &mut env,
        db.clone(),
        "/subscribe/edit".into(),
        vec![],
    );
    let rendered_css = env.render_str(THEME_CSS_STR, ()).unwrap_or_default();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{title} — Resummarized</title>
    <link rel="icon" href="/website/static/logo.svg" type="image/svg+xml">
    <style>{rendered_css}</style>
</head>
<body>
    <nav class="gjs-navbar site-header">
        <div class="container nav-container">
            <a href="/" class="gjs-navbar-brand brand">
                <img alt="Resummarized" class="gjs-navbar-logo brand-logo" src="/website/static/logo.svg">
            </a>
        </div>
    </nav>
    <section class="gjs-cta section subscribe-main">
        <div class="container">
            <div class="gjs-cta-box cta-box">
                <h1 class="gjs-cta-title">{heading}</h1>
                <p class="subscribe-status is-error" style="display:block;">{message}</p>
                <div class="subscribe-row" style="margin-top: 24px;">
                    <a href="/subscribe" class="gjs-button btn btn-primary">Subscribe to Newsletter</a>
                </div>
            </div>
        </div>
    </section>
</body>
</html>"#
    );
    no_cache_html_response(status, html)
}

fn render_subscribe_cancelled_page(db: &sea_orm::DatabaseConnection) -> Response {
    let mut env = minijinja::Environment::new();
    lariv_rs::plugins::website::template_funcs::register_funcs(
        &mut env,
        db.clone(),
        "/subscribe/cancel".into(),
        vec![],
    );
    let rendered_css = env.render_str(THEME_CSS_STR, ()).unwrap_or_default();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Subscription Cancelled — Resummarized</title>
    <link rel="icon" href="/website/static/logo.svg" type="image/svg+xml">
    <style>{rendered_css}</style>
</head>
<body>
    <nav class="gjs-navbar site-header">
        <div class="container nav-container">
            <a href="/" class="gjs-navbar-brand brand">
                <img alt="Resummarized" class="gjs-navbar-logo brand-logo" src="/website/static/logo.svg">
            </a>
        </div>
    </nav>
    <section class="gjs-cta section subscribe-main">
        <div class="container">
            <div class="gjs-cta-box cta-box">
                <h1 class="gjs-cta-title">Subscription Cancelled</h1>
                <p class="subscribe-status is-ok" style="display:block;">Your subscription has been successfully cancelled. You will no longer receive emails from Resummarized.</p>
                <div class="subscribe-row" style="margin-top: 24px;">
                    <a href="/" class="gjs-button btn btn-primary">Return to Homepage</a>
                </div>
            </div>
        </div>
    </section>
</body>
</html>"#
    );
    no_cache_html_response(StatusCode::OK, html)
}

fn render_subscribe_edit_page(
    db: &sea_orm::DatabaseConnection,
    sub: &subscriber::Model,
    form_token: &Uuid,
    status_ok: bool,
    status_error: &str,
) -> Result<Response, minijinja::Error> {
    let mut env = minijinja::Environment::new();
    lariv_rs::plugins::website::template_funcs::register_funcs(
        &mut env,
        db.clone(),
        "/subscribe/edit".into(),
        vec![],
    );
    let rendered_css = env.render_str(THEME_CSS_STR, ())?;

    let filter_exchanges = json_to_string_list(&sub.filter_exchanges);
    let filter_entities = json_to_string_list(&sub.filter_entities);
    let filter_event_types = json_to_string_list(&sub.filter_event_types);

    let ctx = minijinja::context! {
        email => sub.email,
        one_time_token => form_token.to_string(),
        newsletter_interval => sub.newsletter_interval.as_str(),
        filter_exchanges_json => serde_json::to_string(&filter_exchanges).unwrap_or_else(|_| "[]".into()),
        filter_entities_json => serde_json::to_string(&filter_entities).unwrap_or_else(|_| "[]".into()),
        filter_event_types_json => serde_json::to_string(&filter_event_types).unwrap_or_else(|_| "[]".into()),
        theme_css => rendered_css,
        status_ok => status_ok,
        status_error => status_error,
    };
    let html = env.render_str(SUBSCRIBE_EDIT_HTML, ctx)?;
    Ok(no_cache_html_response(StatusCode::OK, html))
}

pub async fn subscribe_edit_get(
    Cap(state): Cap<PublisherState>,
    Query(query): Query<SubscribeEditQuery>,
    headers: axum::http::HeaderMap,
) -> Response {
    let (Some(email_param), Some(token_param)) = (&query.email, &query.one_time_token) else {
        return render_subscribe_error_page(
            &state.db,
            "Access Denied",
            "Missing Link Parameters",
            "This edit subscription link is missing required parameters (email or token). Please use the complete link sent to your email.",
            StatusCode::BAD_REQUEST,
        );
    };

    let email = normalize_subscriber_email(email_param);
    let Ok(provided_token) = Uuid::parse_str(token_param.trim()) else {
        return render_subscribe_error_page(
            &state.db,
            "Invalid Token",
            "Invalid Link Token",
            "The token provided in this link is not formatted correctly. Please check your email for the newest link.",
            StatusCode::BAD_REQUEST,
        );
    };

    let sub = match SubscriberEntity::find()
        .filter(subscriber::Column::Email.eq(&email))
        .one(&state.db)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return render_subscribe_error_page(
                &state.db,
                "Subscriber Not Found",
                "Subscriber Not Found",
                "No subscriber was found for this email address.",
                StatusCode::NOT_FOUND,
            );
        }
        Err(e) => {
            tracing::error!(error = %e, email = %email, "failed to query subscriber");
            return render_subscribe_error_page(
                &state.db,
                "Error",
                "Internal Error",
                "An unexpected database error occurred. Please try again later.",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if sub.one_time_token != Some(provided_token) {
        return render_subscribe_error_page(
            &state.db,
            "Link Expired",
            "Invalid or Expired Link",
            "This link has expired or has already been used. For your security, a new link is generated each time. Please check your email for the newest link.",
            StatusCode::FORBIDDEN,
        );
    }

    // Rate limit check: max 5 per 5 minutes per subscriber
    if !state.rate_limiter.check_link_generation(&email).await {
        return render_subscribe_error_page(
            &state.db,
            "Rate Limit Exceeded",
            "Too Many Link Requests",
            "You have exceeded the limit of 5 link generations per 5 minutes. Please wait a few minutes before trying again.",
            StatusCode::TOO_MANY_REQUESTS,
        );
    }

    // Regenerate token immediately upon opening
    let new_token = Uuid::new_v4();
    let mut am: subscriber::ActiveModel = sub.clone().into();
    am.one_time_token = Set(Some(new_token));
    am.updated_at = Set(Some(Utc::now()));
    let updated_sub = match am.update(&state.db).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "failed to rotate subscriber one_time_token on page open");
            return render_subscribe_error_page(
                &state.db,
                "Error",
                "Internal Error",
                "Failed to update security token. Please try again later.",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    // Send email with the new link asynchronously
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:42069");
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let encoded_email: String = form_urlencoded::byte_serialize(email.as_bytes()).collect();
    let new_link = format!("{proto}://{host}/subscribe/edit?email={encoded_email}&one_time_token={new_token}");

    let db_clone = state.db.clone();
    let email_clone = email.clone();
    let link_clone = new_link.clone();
    tokio::spawn(async move {
        if let Ok(prefs) = load_preferences(&db_clone).await {
            if let Err(e) = send_edit_link_opened_email(&prefs, &email_clone, &link_clone).await {
                tracing::warn!(error = %e, email = %email_clone, "failed to send edit link opened email");
            }
        }
    });

    match render_subscribe_edit_page(&state.db, &updated_sub, &new_token, false, "") {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!(error = %e, "failed to render subscribe_edit template");
            render_subscribe_error_page(
                &state.db,
                "Error",
                "Render Error",
                "Failed to render subscription edit page.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

pub async fn subscribe_edit_post(
    Cap(state): Cap<PublisherState>,
    Cap(markets): Cap<StockMarketRegistry>,
    headers: axum::http::HeaderMap,
    HtmlFormBody(form): HtmlFormBody<PublicEditSubscribeForm>,
) -> Response {
    let email = normalize_subscriber_email(&form.email);
    let Ok(submitted_token) = Uuid::parse_str(form.one_time_token.trim()) else {
        return render_subscribe_error_page(
            &state.db,
            "Invalid Token",
            "Invalid Submission Token",
            "The submission token is invalid.",
            StatusCode::BAD_REQUEST,
        );
    };

    let sub = match SubscriberEntity::find()
        .filter(subscriber::Column::Email.eq(&email))
        .one(&state.db)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return render_subscribe_error_page(
                &state.db,
                "Not Found",
                "Subscriber Not Found",
                "No subscription was found for this email address.",
                StatusCode::NOT_FOUND,
            );
        }
        Err(e) => {
            tracing::error!(error = %e, email = %email, "failed to query subscriber");
            return render_subscribe_error_page(
                &state.db,
                "Error",
                "Internal Error",
                "An unexpected database error occurred. Please try again later.",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if sub.one_time_token != Some(submitted_token) {
        return render_subscribe_error_page(
            &state.db,
            "Token Expired",
            "Invalid or Expired Token",
            "This link has expired or has already been used. Please check your email for the newest link.",
            StatusCode::FORBIDDEN,
        );
    }

    // Rate limit check: max 5 per 5 minutes per subscriber
    if !state.rate_limiter.check_link_generation(&email).await {
        return render_subscribe_error_page(
            &state.db,
            "Rate Limit Exceeded",
            "Too Many Link Requests",
            "You have exceeded the limit of 5 link generations per 5 minutes. Please wait a few minutes before trying again.",
            StatusCode::TOO_MANY_REQUESTS,
        );
    }

    let newsletter_interval = if form.newsletter_interval.trim().is_empty() {
        sub.newsletter_interval
    } else {
        match NewsletterInterval::parse(&form.newsletter_interval) {
            Some(interval) => interval,
            None => {
                return render_subscribe_error_page(
                    &state.db,
                    "Invalid Interval",
                    "Invalid Newsletter Interval",
                    "Please select a valid newsletter cadence (daily, weekly, or monthly).",
                    StatusCode::BAD_REQUEST,
                );
            }
        }
    };

    let filter_exchanges = string_list_to_json({
        let exchanges =
            markets.retain_known(normalize_slug_list(form.filter_exchanges).unwrap_or_default());
        if exchanges.is_empty() {
            None
        } else {
            Some(exchanges)
        }
    });
    let filter_entities = string_list_to_json(normalize_filter_list(form.filter_entities));
    let filter_event_types = string_list_to_json(normalize_slug_list(form.filter_event_types));

    // Regenerate token again upon form submission
    let final_token = Uuid::new_v4();
    let mut am: subscriber::ActiveModel = sub.into();
    am.newsletter_interval = Set(newsletter_interval);
    am.filter_exchanges = Set(filter_exchanges);
    am.filter_entities = Set(filter_entities);
    am.filter_event_types = Set(filter_event_types);
    am.one_time_token = Set(Some(final_token));
    am.updated_at = Set(Some(Utc::now()));

    let updated_sub = match am.update(&state.db).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "failed to update subscriber preferences");
            return render_subscribe_error_page(
                &state.db,
                "Error",
                "Internal Error",
                "Failed to save preferences. Please try again later.",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    // Send confirmation email with the new link asynchronously
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:42069");
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let encoded_email: String = form_urlencoded::byte_serialize(email.as_bytes()).collect();
    let new_link = format!("{proto}://{host}/subscribe/edit?email={encoded_email}&one_time_token={final_token}");

    let db_clone = state.db.clone();
    let email_clone = email.clone();
    let link_clone = new_link.clone();
    tokio::spawn(async move {
        if let Ok(prefs) = load_preferences(&db_clone).await {
            if let Err(e) = send_subscription_updated_email(&prefs, &email_clone, &link_clone).await {
                tracing::warn!(error = %e, email = %email_clone, "failed to send subscription updated email");
            }
        }
    });

    match render_subscribe_edit_page(&state.db, &updated_sub, &final_token, true, "") {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!(error = %e, "failed to render subscribe_edit template on submit");
            render_subscribe_error_page(
                &state.db,
                "Error",
                "Render Error",
                "Failed to render confirmation page.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

pub async fn subscribe_cancel_post(
    Cap(state): Cap<PublisherState>,
    HtmlFormBody(form): HtmlFormBody<PublicCancelSubscribeForm>,
) -> Response {
    let email = normalize_subscriber_email(&form.email);
    let Ok(provided_token) = Uuid::parse_str(form.one_time_token.trim()) else {
        return render_subscribe_error_page(
            &state.db,
            "Invalid Token",
            "Invalid Security Token",
            "The token provided in this cancellation request is not formatted correctly. Please try again with the newest link sent to your email.",
            StatusCode::FORBIDDEN,
        );
    };

    if !state.rate_limiter.check_link_generation(&email).await {
        tracing::warn!(email = %email, "rate limit exceeded for subscriber cancellation request");
        return render_subscribe_error_page(
            &state.db,
            "Rate Limit Exceeded",
            "Too Many Requests",
            "You have exceeded the allowed number of requests. Please wait a few minutes before trying again.",
            StatusCode::TOO_MANY_REQUESTS,
        );
    }

    let sub = match SubscriberEntity::find()
        .filter(subscriber::Column::Email.eq(&email))
        .one(&state.db)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return render_subscribe_error_page(
                &state.db,
                "Subscriber Not Found",
                "Subscriber Not Found",
                "No active subscriber was found for this email address.",
                StatusCode::NOT_FOUND,
            );
        }
        Err(e) => {
            tracing::error!(error = %e, email = %email, "failed to query subscriber on cancel");
            return render_subscribe_error_page(
                &state.db,
                "Error",
                "Internal Error",
                "An unexpected database error occurred. Please try again later.",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if sub.one_time_token != Some(provided_token) {
        tracing::warn!(
            email = %email,
            expected = ?sub.one_time_token,
            got = %provided_token,
            "mismatched one-time-token on subscription cancel"
        );
        return render_subscribe_error_page(
            &state.db,
            "Access Denied",
            "Link Expired or Invalid",
            "This link has expired or has already been used. Please check your email for the most recent link.",
            StatusCode::FORBIDDEN,
        );
    }

    if let Err(e) = sub.delete(&state.db).await {
        tracing::error!(error = %e, email = %email, "failed to delete subscriber on cancel");
        return render_subscribe_error_page(
            &state.db,
            "Error",
            "Internal Error",
            "Failed to cancel subscription. Please try again later.",
            StatusCode::INTERNAL_SERVER_ERROR,
        );
    }

    render_subscribe_cancelled_page(&state.db)
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
        filter_exchanges: json_to_string_list(&m.filter_exchanges),
        filter_entities: json_to_string_list(&m.filter_entities),
        filter_event_types: json_to_string_list(&m.filter_event_types),
        newsletter_interval: m.newsletter_interval.as_str().to_string(),
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
        filter_exchanges: form.filter_exchanges.clone(),
        filter_entities: form.filter_entities.clone(),
        filter_event_types: form.filter_event_types.clone(),
        newsletter_interval: form.newsletter_interval.clone(),
        error,
    }
}

pub async fn edit_post(
    Cap(state): Cap<PublisherState>,
    Cap(markets): Cap<StockMarketRegistry>,
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
    let (filter_exchanges, filter_entities, filter_event_types, newsletter_interval) =
        match parsed_subscriber_filters(
            form.filter_exchanges.clone(),
            form.filter_entities.clone(),
            form.filter_event_types.clone(),
            &form.newsletter_interval,
            &markets,
        ) {
            Ok(parsed) => parsed,
            Err(msg) => {
                return html_built_page_with_slots(
                    &edit_modal_from_form(id, &form, q.form_name(), msg),
                    &chrome,
                    &SlotCtx::from_auth(&ctx),
                )
                .into_response();
            }
        };
    let now = Utc::now();
    let mut am: subscriber::ActiveModel = existing.into();
    am.updated_at = Set(Some(now));
    am.email = Set(email);
    am.subscription_date = Set(subscription_date);
    am.filter_exchanges = Set(filter_exchanges);
    am.filter_entities = Set(filter_entities);
    am.filter_event_types = Set(filter_event_types);
    am.newsletter_interval = Set(newsletter_interval);
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

#[derive(Debug, serde::Deserialize, Default)]
pub struct BulkIdsQuery {
    #[serde(default)]
    pub ids: Option<String>,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct BulkIdsForm {
    #[serde(default)]
    pub ids: String,
}

fn ids_csv(ids: &[i64]) -> String {
    ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn bulk_delete_message(count: usize) -> String {
    if count == 1 {
        "Are you sure you want to delete the selected subscriber?".into()
    } else {
        format!("Are you sure you want to delete {count} selected subscribers?")
    }
}

fn bulk_delete_page(ids: &[i64], error: String, can_submit: bool) -> ConfirmBulkDeletePage {
    ConfirmBulkDeletePage {
        modal_uid: SubscriberBulkDeleteModalKey::ID.to_string(),
        message: if ids.is_empty() {
            "Select at least one subscriber to delete.".into()
        } else {
            bulk_delete_message(ids.len())
        },
        form_name: BULK_DELETE_FORM.into(),
        ids: ids_csv(ids),
        error,
        can_submit,
    }
}

pub async fn bulk_delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<BulkIdsQuery>,
) -> maud::Markup {
    if !ctx.user.is_superuser {
        return maud::html! { div class="alert alert-error" { "Forbidden" } };
    }
    let ids = parse_bulk_ids(q.ids.as_deref().unwrap_or(""));
    let (error, can_submit) = if ids.is_empty() {
        ("No subscribers selected.".into(), false)
    } else {
        (String::new(), true)
    };
    html_built_page_with_slots(
        &bulk_delete_page(&ids, error, can_submit),
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn bulk_delete_post(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    HtmlFormBody(form): HtmlFormBody<BulkIdsForm>,
) -> Response {
    let list_url = list_url();
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url).into_response();
    }
    let ids = parse_bulk_ids(&form.ids);
    if ids.is_empty() {
        return html_built_page_with_slots(
            &bulk_delete_page(&ids, "No subscribers selected.".into(), false),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }
    match SubscriberEntity::delete_many()
        .filter(subscriber::Column::Id.is_in(ids.clone()))
        .exec(&state.db)
        .await
    {
        Ok(res) if res.rows_affected == 0 => html_built_page_with_slots(
            &bulk_delete_page(
                &ids,
                "None of the selected subscribers could be found.".into(),
                true,
            ),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response(),
        Ok(_) => htmx.redirect(&list_url),
        Err(e) => {
            tracing::error!(error = %e, "failed to bulk-delete subscribers");
            html_built_page_with_slots(
                &bulk_delete_page(&ids, e.to_string(), true),
                &chrome,
                &SlotCtx::from_auth(&ctx),
            )
            .into_response()
        }
    }
}

fn build_subscriber_edit_link(headers: &header::HeaderMap, email: &str, token: &Uuid) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:42069");
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let encoded_email: String = form_urlencoded::byte_serialize(email.as_bytes()).collect();
    format!("{proto}://{host}/subscribe/edit?email={encoded_email}&one_time_token={token}")
}

fn send_edit_link_modal(id: i64, email: String, error: String) -> SubscriberSendEditLinkModalPage {
    SubscriberSendEditLinkModalPage {
        modal_uid: SubscriberSendEditLinkModalKey::ID.to_string(),
        subscriber_id: id,
        email,
        error,
    }
}

pub async fn send_edit_link_get(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> maud::Markup {
    if !ctx.user.is_superuser {
        return maud::html! { div class="alert alert-error" { "Forbidden" } };
    }
    let Some(sub) = find_subscriber_scoped(&state.db, id, &ctx).await else {
        return maud::html! { div class="alert alert-error" { "Subscriber not found" } };
    };
    html_built_page_with_slots(
        &send_edit_link_modal(id, sub.email, String::new()),
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn send_edit_link_post(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url()).into_response();
    }
    let Some(sub) = find_subscriber_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&list_url()).into_response();
    };

    let new_token = Uuid::new_v4();
    let mut am: subscriber::ActiveModel = sub.clone().into();
    am.one_time_token = Set(Some(new_token));
    am.updated_at = Set(Some(Utc::now()));
    if let Err(e) = am.update(&state.db).await {
        tracing::error!(error = %e, id, "failed to rotate subscriber token on send_edit_link_post");
        return html_built_page_with_slots(
            &send_edit_link_modal(id, sub.email, "Failed to update security token.".into()),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }

    let edit_link = build_subscriber_edit_link(&headers, &sub.email, &new_token);
    let prefs = match load_preferences(&state.db).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "failed to load publisher preferences");
            return html_built_page_with_slots(
                &send_edit_link_modal(id, sub.email, "Failed to load preferences.".into()),
                &chrome,
                &SlotCtx::from_auth(&ctx),
            )
            .into_response();
        }
    };

    let email = sub.email.clone();
    tokio::spawn(async move {
        if let Err(e) = send_change_subscription_email(&prefs, &email, &edit_link).await {
            tracing::warn!(error = %e, email = %email, "failed to send change subscription email");
        }
    });

    htmx.redirect(&SubscriberDetailRouteTag::new(id).url())
}

fn bulk_send_edit_link_page(ids: &[i64], error: String, can_submit: bool) -> SubscriberBulkSendEditLinkModalPage {
    SubscriberBulkSendEditLinkModalPage {
        modal_uid: SubscriberBulkSendEditLinkModalKey::ID.to_string(),
        ids: ids_csv(ids),
        recipient_count: ids.len(),
        error,
        can_submit,
    }
}

pub async fn bulk_send_edit_link_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<BulkIdsQuery>,
) -> maud::Markup {
    if !ctx.user.is_superuser {
        return maud::html! { div class="alert alert-error" { "Forbidden" } };
    }
    let ids = parse_bulk_ids(q.ids.as_deref().unwrap_or(""));
    let (error, can_submit) = if ids.is_empty() {
        ("No subscribers selected.".into(), false)
    } else {
        (String::new(), true)
    };
    html_built_page_with_slots(
        &bulk_send_edit_link_page(&ids, error, can_submit),
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn bulk_send_edit_link_post(
    Cap(state): Cap<PublisherState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    headers: axum::http::HeaderMap,
    HtmlFormBody(form): HtmlFormBody<BulkIdsForm>,
) -> Response {
    let list_url = list_url();
    if !ctx.user.is_superuser {
        return Redirect::to(&list_url).into_response();
    }
    let ids = parse_bulk_ids(&form.ids);
    if ids.is_empty() {
        return html_built_page_with_slots(
            &bulk_send_edit_link_page(&ids, "No subscribers selected.".into(), false),
            &chrome,
            &SlotCtx::from_auth(&ctx),
        )
        .into_response();
    }

    let subscribers = match SubscriberEntity::find()
        .filter(subscriber::Column::Id.is_in(ids.clone()))
        .all(&state.db)
        .await
    {
        Ok(subs) => subs,
        Err(e) => {
            tracing::error!(error = %e, "failed to query subscribers for bulk edit link send");
            return html_built_page_with_slots(
                &bulk_send_edit_link_page(&ids, "Database query failed.".into(), true),
                &chrome,
                &SlotCtx::from_auth(&ctx),
            )
            .into_response();
        }
    };

    let prefs = match load_preferences(&state.db).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "failed to load publisher preferences");
            return html_built_page_with_slots(
                &bulk_send_edit_link_page(&ids, "Failed to load preferences.".into(), true),
                &chrome,
                &SlotCtx::from_auth(&ctx),
            )
            .into_response();
        }
    };

    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:42069")
        .to_string();
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http")
        .to_string();

    let mut send_tasks = Vec::new();
    for sub in subscribers {
        let new_token = Uuid::new_v4();
        let mut am: subscriber::ActiveModel = sub.clone().into();
        am.one_time_token = Set(Some(new_token));
        am.updated_at = Set(Some(Utc::now()));
        if let Err(e) = am.update(&state.db).await {
            tracing::warn!(error = %e, subscriber_id = sub.id, "failed to rotate subscriber token for bulk link send");
            continue;
        }
        let encoded_email: String = form_urlencoded::byte_serialize(sub.email.as_bytes()).collect();
        let edit_link = format!("{proto}://{host}/subscribe/edit?email={encoded_email}&one_time_token={new_token}");
        send_tasks.push((sub.email, edit_link));
    }

    tokio::spawn(async move {
        for (email, edit_link) in send_tasks {
            if let Err(e) = send_change_subscription_email(&prefs, &email, &edit_link).await {
                tracing::warn!(error = %e, email = %email, "failed to send bulk change subscription email");
            }
        }
    });

    htmx.redirect(&list_url)
}

fn prefs_page(prefs: PublisherPreferences, error: String) -> PublisherPreferencesPage {
    PublisherPreferencesPage {
        html_template: prefs.html_template,
        smtp_host: prefs.smtp_host,
        smtp_port: prefs.smtp_port,
        smtp_username: prefs.smtp_username,
        smtp_password: prefs.smtp_password,
        smtp_from: prefs.smtp_from,
        edit_link_email_subject: prefs.edit_link_email_subject,
        edit_link_email_template: prefs.edit_link_email_template,
        edit_opened_email_subject: prefs.edit_opened_email_subject,
        edit_opened_email_template: prefs.edit_opened_email_template,
        edit_updated_email_subject: prefs.edit_updated_email_subject,
        edit_updated_email_template: prefs.edit_updated_email_template,
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
        edit_link_email_subject: form.edit_link_email_subject,
        edit_link_email_template: form.edit_link_email_template,
        edit_opened_email_subject: form.edit_opened_email_subject,
        edit_opened_email_template: form.edit_opened_email_template,
        edit_updated_email_subject: form.edit_updated_email_subject,
        edit_updated_email_template: form.edit_updated_email_template,
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
