use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    html_form::HtmlFormBody,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    template::RenderAppPane,
    web::{
        Htmx, ModalFormQuery as ModalNameQuery, QueryPage, html_built_page_or_app_layout,
        html_built_page_with_slots, respond_create_modal_done, respond_edit_modal_done,
    },
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait};

use super::{
    entities::subscriber::{self, Entity as SubscriberEntity},
    forms::SubscriberForm,
    keys::{
        SubscriberCreateModalKey, SubscriberDeleteModalKey, SubscriberEditModalKey,
        SubscriberTableKey,
    },
    routes::{SubscriberDefaultRouteTag, SubscriberDeletePostRouteTag, SubscriberDetailRouteTag},
    scope::{
        apply_email_filter, apply_subscriber_sort, email_in_use, find_subscriber_scoped,
        scope_superuser,
    },
    state::PublisherState,
    templates::{
        ConfirmDeletePage, SubscriberCreateModalPage, SubscriberDetailPage,
        SubscriberEditModalPage, SubscriberListPage, SubscriberRow,
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
    let email = form.email.trim().to_string();
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
    let email = form.email.trim().to_string();
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
