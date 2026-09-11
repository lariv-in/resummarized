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
    extras,
    feeds::{BseFeedKind, item_document_link},
    fetch,
    keys::ItemTableKey,
    routes::{default_feed_url, feed_list_url},
    state::BseState,
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
    Cap(state): Cap<BseState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Path(feed): Path<String>,
    Query(q): Query<ItemListQuery>,
) -> Response {
    let Some(kind) = BseFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };

    let show_description = kind.shows_description_column();
    let sort = q.sort.as_deref().unwrap_or("").trim();
    let query = extras::apply_sort(
        extras::apply_filters(extras::list_select(kind), kind, &q.filters),
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
    let ids: Vec<i64> = models.iter().map(|m| m.id).collect();
    let extras_map = extras::load_map(&state.db, kind, &ids)
        .await
        .unwrap_or_default();

    let rows: Vec<FeedItemRow> = models
        .into_iter()
        .map(|m| {
            let extra = extras::extra_cells(kind, extras_map.get(&m.id), &ctx.timezone)
                .into_iter()
                .map(|s| truncate(&s, 140))
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

    let extra_columns: Vec<FeedListColumn> = extras::extra_columns(kind)
        .into_iter()
        .map(|f| FeedListColumn {
            sort_key: f.key,
            label: f.label,
        })
        .collect();

    let page = FeedItemListPage {
        feed_slug: kind.slug().to_string(),
        feed_name: kind.display_name().to_string(),
        extra_fields: extra_columns,
        show_description,
        items: ObjectList::from_page(rows, page_num, DEFAULT_PAGE_SIZE, total),
        sort: q.sort.clone().unwrap_or_default(),
        filter_fields: extras::filter_fields(kind),
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
    Cap(state): Cap<BseState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path((feed, id)): Path<(String, i64)>,
) -> Response {
    let Some(kind) = BseFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };
    let Some(item) = lariv_rs::web::opt_or_log(
        ItemEntity::find_by_id(id).one(&state.db).await,
        "find bse item by id",
    ) else {
        return Redirect::to(&feed_list_url(kind.slug())).into_response();
    };
    if item.feed_kind != kind.slug() {
        return Redirect::to(&feed_list_url(kind.slug())).into_response();
    }
    let extra = extras::load_map(&state.db, kind, &[item.id])
        .await
        .ok()
        .and_then(|mut map| map.remove(&item.id));
    let extra_fields = extras::detail_fields(extra.as_ref(), &ctx.timezone);
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
    Cap(state): Cap<BseState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(feed): Path<String>,
) -> Response {
    let Some(kind) = BseFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };
    if let Err(e) = fetch::fetch_feed(&state, kind).await {
        tracing::warn!(feed = kind.slug(), error = %e, "manual BSE refresh failed");
    }
    htmx.redirect(&feed_list_url(kind.slug()))
}
