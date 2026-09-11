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
    feeds::{JpxFeedKind, item_document_link},
    fetch,
    keys::ItemTableKey,
    query,
    routes::{default_feed_url, feed_list_url},
    state::JpxState,
    templates::{FeedItemDetailPage, FeedItemListPage, FeedItemRow},
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

fn format_datetime(dt: Option<chrono::DateTime<chrono::Utc>>, tz: &str) -> String {
    dt.map(|d| lariv_rs::datetime::DatetimeLabel::seconds(d, tz).into_string())
        .unwrap_or_default()
}

pub async fn list(
    Cap(state): Cap<JpxState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Path(feed): Path<String>,
    Query(q): Query<ItemListQuery>,
) -> Response {
    let Some(kind) = JpxFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };

    let sort = q.sort.as_deref().unwrap_or("").trim();
    let query = query::apply_sort(
        query::apply_filters(query::list_select(kind), &q.filters),
        sort,
    );

    let page_num = q.page.get();
    let paginator = query.paginate(&state.db, DEFAULT_PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_num as u64).saturating_sub(1))
        .await
        .unwrap_or_default();

    let rows: Vec<FeedItemRow> = models
        .into_iter()
        .map(|m| FeedItemRow {
            id: m.id,
            title: m.title,
            pub_date: format_datetime(m.pub_date, &ctx.timezone),
        })
        .collect();

    let page = FeedItemListPage {
        feed_slug: kind.slug().to_string(),
        feed_name: kind.display_name().to_string(),
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
    Cap(state): Cap<JpxState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path((feed, id)): Path<(String, i64)>,
) -> Response {
    let Some(kind) = JpxFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };
    let Some(item) = lariv_rs::web::opt_or_log(
        ItemEntity::find_by_id(id).one(&state.db).await,
        "find jpx item by id",
    ) else {
        return Redirect::to(&feed_list_url(kind.slug())).into_response();
    };
    if item.feed_kind != kind.slug() {
        return Redirect::to(&feed_list_url(kind.slug())).into_response();
    }
    let mut extra_fields = Vec::new();
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
    Cap(state): Cap<JpxState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(feed): Path<String>,
) -> Response {
    let Some(kind) = JpxFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };
    if let Err(e) = fetch::fetch_feed(&state, kind).await {
        tracing::warn!(feed = kind.slug(), error = %e, "manual JPX refresh failed");
    }
    htmx.redirect(&feed_list_url(kind.slug()))
}
