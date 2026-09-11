use std::collections::HashMap;

use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    template::RenderAppPane,
    web::{Htmx, QueryPage, html_built_page_or_app_layout, html_built_page_with_slots},
};
use sea_orm::{EntityTrait, PaginatorTrait};
use serde::Deserialize;

use super::{
    entities::item::Entity as ItemEntity,
    feeds::{NasdaqFeedKind, item_document_link},
    fetch,
    keys::ItemTableKey,
    query,
    routes::{default_feed_url, feed_list_url},
    state::NasdaqState,
    templates::{FeedItemDetailPage, FeedItemListPage, FeedItemRow, FeedListColumn},
};

#[derive(Debug, Deserialize, Default)]
pub struct ItemListQuery {
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: QueryPage,
    #[serde(flatten)]
    pub filters: HashMap<String, String>,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

fn truncate(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let taken: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{taken}…")
    } else {
        taken
    }
}

fn format_datetime(dt: Option<chrono::DateTime<chrono::Utc>>, tz: &str) -> String {
    dt.map(|d| lariv_rs::datetime::DatetimeLabel::seconds(d, tz).into_string())
        .unwrap_or_default()
}

pub async fn list(
    Cap(state): Cap<NasdaqState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Path(feed): Path<String>,
    Query(q): Query<ItemListQuery>,
) -> Response {
    let Some(kind) = NasdaqFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };

    let show_description = kind.shows_description_column();
    let sort = q.sort.as_deref().unwrap_or("").trim();
    let query = query::apply_sort(
        query::apply_filters(query::list_select(kind), kind, &q.filters),
        kind,
        sort,
    );

    let page_num = q.page.get();
    let paginator = query.paginate(&state.db, DEFAULT_PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_num as u64).saturating_sub(1))
        .await
        .unwrap_or_default();

    let extra_columns: Vec<FeedListColumn> = kind
        .extra_list_fields()
        .iter()
        .map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        })
        .collect();

    let rows: Vec<FeedItemRow> = models
        .into_iter()
        .map(|m| {
            let extra = extra_columns
                .iter()
                .map(|col| match col.sort_key {
                    "Category" => truncate(m.category.as_deref().unwrap_or(""), 140),
                    _ => String::new(),
                })
                .collect();
            FeedItemRow {
                extra,
                id: m.id,
                title: m.title,
                pub_date: format_datetime(m.pub_date, &ctx.timezone),
                description: truncate(&m.description, 140),
            }
        })
        .collect();

    let page = FeedItemListPage {
        feed_slug: kind.slug().to_string(),
        feed_name: kind.display_name().to_string(),
        extra_fields: extra_columns,
        show_description,
        items: ObjectList::from_page(rows, page_num, DEFAULT_PAGE_SIZE, total),
        sort: q.sort.clone().unwrap_or_default(),
        filter_fields: query::filter_fields(kind),
        filter_values: q.filters,
        path_and_query: path_and_query(&uri),
    };
    if htmx.targets::<ItemTableKey>() {
        return page.render_table().into_response();
    }
    if htmx.wants_main_content() {
        let markup: maud::Markup = page.render_main().into();
        return markup.into_response();
    }
    if htmx.wants_app_layout() {
        let markup: maud::Markup = page.render_pane().into();
        return markup.into_response();
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn detail(
    Cap(state): Cap<NasdaqState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path((feed, id)): Path<(String, i64)>,
) -> Response {
    let Some(kind) = NasdaqFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };
    let Some(item) = lariv_rs::web::opt_or_log(
        ItemEntity::find_by_id(id).one(&state.db).await,
        "find nasdaq item by id",
    ) else {
        return Redirect::to(&feed_list_url(kind.slug())).into_response();
    };
    if item.feed_kind != kind.slug() {
        return Redirect::to(&feed_list_url(kind.slug())).into_response();
    }
    let mut extra_fields = Vec::new();
    if let Some(category) = item.category.as_deref().filter(|s| !s.is_empty()) {
        extra_fields.push(("Category".to_string(), category.to_string()));
    }
    if let Some(author) = item.author.as_deref().filter(|s| !s.is_empty()) {
        extra_fields.push(("Author".to_string(), author.to_string()));
    }
    if let Some(guid) = item.guid.as_deref().filter(|s| !s.is_empty()) {
        extra_fields.push(("Guid".to_string(), guid.to_string()));
    }
    let page = FeedItemDetailPage {
        feed_slug: kind.slug().to_string(),
        feed_name: kind.display_name().to_string(),
        title: item.title,
        link: item_document_link(&item.link).map(str::to_string),
        description: item.description,
        pub_date: format_datetime(item.pub_date, &ctx.timezone),
        extra_fields,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn refresh(
    Cap(state): Cap<NasdaqState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(feed): Path<String>,
) -> Response {
    let Some(kind) = NasdaqFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };
    if let Err(e) = fetch::fetch_feed(&state, kind).await {
        tracing::warn!(feed = kind.slug(), error = %e, "manual Nasdaq refresh failed");
    }
    htmx.redirect(&feed_list_url(kind.slug()))
}
