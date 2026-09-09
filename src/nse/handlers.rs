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
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use super::{
    brsr::BrsrField,
    description::DescriptionField,
    entities::item::{self, Entity as ItemEntity},
    feeds::{NseFeedKind, item_document_link},
    fetch,
    fr::FrField,
    ic::IcField,
    iff::IffField,
    it::ItField,
    keys::ItemTableKey,
    routes::{default_feed_url, feed_list_url},
    rpt::RptField,
    scr::ScrField,
    shp::ShpField,
    sod::{self, SodField},
    state::NseState,
    templates::{FeedItemDetailPage, FeedItemListPage, FeedItemRow, FeedListColumn},
    uhp::UhpField,
    voting::VoteField,
};

#[derive(Debug, Deserialize, Default)]
pub struct ItemListQuery {
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

fn sort_direction(sort: &str, key: &str) -> Option<bool> {
    let sort = sort.trim();
    let desc = format!("{key} DESC");
    let asc = format!("{key} ASC");
    if sort.eq_ignore_ascii_case(&desc) {
        Some(true)
    } else if sort.eq_ignore_ascii_case(&asc) || sort.eq_ignore_ascii_case(key) {
        Some(false)
    } else {
        None
    }
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
    Cap(state): Cap<NseState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Path(feed): Path<String>,
    Query(q): Query<ItemListQuery>,
) -> Response {
    let Some(kind) = NseFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };

    let extra_fields = kind.extra_list_fields();
    let brsr_fields: &[BrsrField] = if kind == NseFeedKind::Brsr {
        BrsrField::LIST
    } else {
        &[]
    };
    let vote_fields: &[VoteField] = if kind == NseFeedKind::VotingResults {
        VoteField::LIST
    } else {
        &[]
    };
    let uhp_fields: &[UhpField] = if kind == NseFeedKind::UnitholdingPatterns {
        UhpField::LIST
    } else {
        &[]
    };
    let sod_fields: &[SodField] = if kind == NseFeedKind::StatementOfDeviation {
        SodField::LIST
    } else {
        &[]
    };
    let shp_fields: &[ShpField] = if kind == NseFeedKind::ShareholdingPattern {
        ShpField::LIST
    } else {
        &[]
    };
    let scr_fields: &[ScrField] = if kind == NseFeedKind::SecretarialCompliance {
        ScrField::LIST
    } else {
        &[]
    };
    let rpt_fields: &[RptField] = if kind == NseFeedKind::RelatedPartyTransactions {
        RptField::LIST
    } else {
        &[]
    };
    let ic_fields: &[IcField] = if kind == NseFeedKind::InvestorComplaints {
        IcField::LIST
    } else {
        &[]
    };
    let it_fields: &[ItField] = if kind == NseFeedKind::InsiderTrading {
        ItField::LIST
    } else {
        &[]
    };
    let iff_fields: &[IffField] = if kind == NseFeedKind::IntegratedFilingFinancials {
        IffField::LIST
    } else {
        &[]
    };
    let fr_fields: &[FrField] = if kind == NseFeedKind::FinancialResults {
        FrField::LIST
    } else {
        &[]
    };
    let show_description = kind.shows_description_column();
    let mut query = ItemEntity::find().filter(item::Column::FeedKind.eq(kind.slug()));
    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = if let Some(desc) = sort_direction(sort, "Title") {
        if desc {
            query.order_by_desc(item::Column::Title)
        } else {
            query.order_by_asc(item::Column::Title)
        }
    } else if let Some(desc) = sort_direction(sort, "PubDate") {
        if desc {
            query.order_by_desc(item::Column::PubDate)
        } else {
            query.order_by_asc(item::Column::PubDate)
        }
    } else if let Some((desc, field)) = extra_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = brsr_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = vote_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = uhp_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = sod_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = shp_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = scr_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = rpt_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = ic_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = it_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = iff_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else if let Some((desc, field)) = fr_fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        if desc {
            query.order_by_desc(field.column())
        } else {
            query.order_by_asc(field.column())
        }
    } else {
        query.order_by_desc(item::Column::Id)
    };

    let page_num = q.page.get();
    let paginator = query.paginate(&state.db, DEFAULT_PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_num as u64).saturating_sub(1))
        .await
        .unwrap_or_default();

    let rows: Vec<FeedItemRow> = models
        .into_iter()
        .map(|m| {
            let mut extra: Vec<String> = extra_fields
                .iter()
                .map(|f| truncate(&f.display(&m, &ctx.timezone), 140))
                .collect();
            extra.extend(brsr_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(vote_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(uhp_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(sod_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(shp_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(scr_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(rpt_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(ic_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(it_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(iff_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            extra.extend(fr_fields.iter().map(|f| truncate(&f.display(&m), 140)));
            FeedItemRow {
                extra,
                id: m.id,
                title: m.title,
                pub_date: format_datetime(m.pub_date, &ctx.timezone),
                description: truncate(&m.description, 140),
            }
        })
        .collect();

    let extra_columns: Vec<FeedListColumn> = extra_fields
        .iter()
        .map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        })
        .chain(brsr_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(vote_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(uhp_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(sod_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(shp_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(scr_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(rpt_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(ic_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(it_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(iff_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .chain(fr_fields.iter().map(|f| FeedListColumn {
            sort_key: f.sort_key(),
            label: f.label(),
        }))
        .collect();

    let page = FeedItemListPage {
        feed_slug: kind.slug().to_string(),
        feed_name: kind.display_name().to_string(),
        extra_fields: extra_columns,
        show_description,
        items: ObjectList::from_page(rows, page_num, DEFAULT_PAGE_SIZE, total),
        sort: q.sort.clone().unwrap_or_default(),
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
    Cap(state): Cap<NseState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path((feed, id)): Path<(String, i64)>,
) -> Response {
    let Some(kind) = NseFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };
    let Some(item) = lariv_rs::web::opt_or_log(
        ItemEntity::find_by_id(id).one(&state.db).await,
        "find nse item by id",
    ) else {
        return Redirect::to(&feed_list_url(kind.slug())).into_response();
    };
    if item.feed_kind != kind.slug() {
        return Redirect::to(&feed_list_url(kind.slug())).into_response();
    }
    let mut extra_fields: Vec<(String, String)> = DescriptionField::ALL
        .iter()
        .filter_map(|f| {
            let value = f.display(&item, &ctx.timezone);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        })
        .collect();
    if kind == NseFeedKind::Brsr {
        extra_fields.extend(BrsrField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::VotingResults {
        extra_fields.extend(VoteField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::UnitholdingPatterns {
        extra_fields.extend(UhpField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::ShareholdingPattern {
        extra_fields.extend(ShpField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::SecretarialCompliance {
        extra_fields.extend(ScrField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::RelatedPartyTransactions {
        extra_fields.extend(RptField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::InvestorComplaints {
        extra_fields.extend(IcField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::InsiderTrading {
        extra_fields.extend(ItField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::IntegratedFilingFinancials {
        extra_fields.extend(IffField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    if kind == NseFeedKind::FinancialResults {
        extra_fields.extend(FrField::DETAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    let mut extra_after = Vec::new();
    let mut objects_table = Vec::new();
    if kind == NseFeedKind::StatementOfDeviation {
        extra_fields.extend(SodField::DETAIL_HEAD.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
        objects_table = sod::parse_object_rows(&item.sod_objects);
        extra_after.extend(SodField::DETAIL_TAIL.iter().filter_map(|f| {
            let value = f.display(&item);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        }));
    }
    let page = FeedItemDetailPage {
        feed_slug: kind.slug().to_string(),
        feed_name: kind.display_name().to_string(),
        title: item.title,
        link: item_document_link(&item.link).map(str::to_string),
        description: item.description,
        pub_date: format_datetime(item.pub_date, &ctx.timezone),
        extra_fields,
        objects_table,
        extra_after,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn refresh(
    Cap(state): Cap<NseState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(feed): Path<String>,
) -> Response {
    let Some(kind) = NseFeedKind::from_slug(&feed) else {
        return Redirect::to(&default_feed_url()).into_response();
    };
    if let Err(e) = fetch::fetch_feed(&state, kind).await {
        tracing::warn!(feed = kind.slug(), error = %e, "manual NSE refresh failed");
    }
    htmx.redirect(&feed_list_url(kind.slug()))
}
