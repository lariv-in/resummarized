//! Idempotent seed for the Resummarized public homepage, subscribe page, static media, and Custom theme.
//!
//! Registered as a [`lariv_rs::hooks::RunSeed`] hook so it runs only for `seed`, not `serve`.

use chrono::Utc;
use lariv_rs::app::MountedApp;
use lariv_rs::hooks::RunSeed;
use lariv_rs::plugin_install::define_plugin_install;
use lariv_rs::plugins::filesystem::node::{self, NodeFile};
use lariv_rs::plugins::filesystem::storage::DynFilestore;
use lariv_rs::plugins::website::{
    WebsiteTag,
    builder_assets::public_asset_url,
    entities::{
        WebsitePreferences,
        db_route::{self, Column as DbRouteColumn, Entity as DbRouteEntity},
    },
    preferences::{self, CUSTOM_THEME_ID},
    render,
    state::WebsiteState,
    template_funcs,
};
use lariv_rs::traits::get::GetByTag;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tokio::io::AsyncReadExt;

/// Hook identity for the deployment-local website seed (distinct from [`WebsiteTag`] state).
pub struct ResummarizedWebsiteSeedTag;

define_plugin_install! {
    plugin: ResummarizedWebsiteSeedTag;
    /// Queue homepage/media seed for the `seed` CLI command.
    steps: [seeds(SeedsHook)]
}

/// Runs [`ensure_homepage`] when seed hooks execute.
#[derive(Clone, Copy, Default)]
pub struct SeedsHook;

#[async_trait::async_trait]
impl<M, WebsiteIdx> RunSeed<M, WebsiteIdx> for SeedsHook
where
    M: GetByTag<WebsiteTag, WebsiteIdx, Value = WebsiteState> + Sync,
{
    async fn run_seed(app: &MountedApp<M>) -> anyhow::Result<()> {
        tracing::info!("resummarized website: seeding homepage, subscribe page, and media");
        ensure_homepage(app.get_capability_output::<WebsiteTag, WebsiteIdx>()).await?;
        tracing::info!("resummarized website: seed complete");
        Ok(())
    }
}

const HOMEPAGE_HTML: &str = include_str!("../assets/homepage.html");
const SUBSCRIBE_HTML: &str = include_str!("../assets/subscribe.html");
const THEME_CSS: &[u8] = include_bytes!("../assets/theme/resummarized.css");
const THEME_JS: &[u8] = include_bytes!("../assets/theme/resummarized.js");
const ROUTE_PATH: &str = "/";
const PAGE_NAME: &str = "index.html";
const SUBSCRIBE_ROUTE_PATH: &str = "/subscribe";
const SUBSCRIBE_PAGE_NAME: &str = "subscribe.html";
const THEME_CSS_NAME: &str = "resummarized.css";
const THEME_JS_NAME: &str = "resummarized.js";
const THEME: &str = CUSTOM_THEME_ID;

struct StaticAsset {
    name: &'static str,
    bytes: &'static [u8],
}

const STATIC_ASSETS: &[StaticAsset] = &[
    StaticAsset {
        name: "logo.svg",
        bytes: include_bytes!("../assets/static/logo.svg"),
    },
    StaticAsset {
        name: "Newsreader-Roman.woff2",
        bytes: include_bytes!("../assets/theme/fonts/Newsreader-Roman.woff2"),
    },
    StaticAsset {
        name: "Newsreader-Italic.woff2",
        bytes: include_bytes!("../assets/theme/fonts/Newsreader-Italic.woff2"),
    },
    StaticAsset {
        name: "Inter-Roman.woff2",
        bytes: include_bytes!("../assets/theme/fonts/Inter-Roman.woff2"),
    },
    StaticAsset {
        name: "Inter-Italic.woff2",
        bytes: include_bytes!("../assets/theme/fonts/Inter-Italic.woff2"),
    },
    StaticAsset {
        name: "JunicodeVF-Roman.woff2",
        bytes: include_bytes!("../assets/theme/fonts/JunicodeVF-Roman.woff2"),
    },
    StaticAsset {
        name: "JunicodeVF-Italic.woff2",
        bytes: include_bytes!("../assets/theme/fonts/JunicodeVF-Italic.woff2"),
    },
    StaticAsset {
        name: "MonaspaceRadonVar.woff2",
        bytes: include_bytes!("../assets/theme/fonts/MonaspaceRadonVar.woff2"),
    },
];

pub async fn ensure_homepage(state: &WebsiteState) -> anyhow::Result<()> {
    ensure_homepage_state(&state.db, state.store.as_ref()).await
}

async fn ensure_homepage_state(
    db: &DatabaseConnection,
    store: &DynFilestore,
) -> anyhow::Result<()> {
    ensure_static_assets(db, store).await?;
    remove_legacy_static_routes(db).await?;
    for (label, source) in [
        ("homepage.html", HOMEPAGE_HTML),
        ("subscribe.html", SUBSCRIBE_HTML),
    ] {
        reject_nonportable_asset_urls(label, source)?;
    }
    // Theme CSS is injected after minijinja render, so resolve media_url at seed.
    let css = render_seeded_template(db, std::str::from_utf8(THEME_CSS)?)?;
    ensure_custom_theme(db, store, css.as_bytes()).await?;
    let (page, page_rewritten) =
        ensure_page_vnode(db, store, PAGE_NAME, HOMEPAGE_HTML.as_bytes()).await?;
    ensure_db_route(db, ROUTE_PATH, page.id, THEME, page_rewritten).await?;
    tracing::info!(
        page_id = page.id,
        "resummarized website: homepage route ready"
    );
    let (subscribe_page, subscribe_rewritten) =
        ensure_page_vnode(db, store, SUBSCRIBE_PAGE_NAME, SUBSCRIBE_HTML.as_bytes()).await?;
    ensure_db_route(
        db,
        SUBSCRIBE_ROUTE_PATH,
        subscribe_page.id,
        THEME,
        subscribe_rewritten,
    )
    .await?;
    tracing::info!(
        page_id = subscribe_page.id,
        "resummarized website: subscribe route ready"
    );
    Ok(())
}

/// Seeds theme CSS/JS under `website/themes/` and points Custom theme preferences at them.
async fn ensure_custom_theme(
    db: &DatabaseConnection,
    store: &DynFilestore,
    css: &[u8],
) -> anyhow::Result<()> {
    let segments = ["website".into(), "themes".into()];
    let parent_id = node::ensure_directory_path(db, store, None, &segments)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let parent = match parent_id {
        Some(id) => match node::get_by_id(db, id).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "get node by id for website themes parent");
                None
            }
        },
        None => None,
    };

    let css = ensure_file_vnode(db, store, parent_id, parent.as_ref(), THEME_CSS_NAME, css)
        .await?
        .0;
    let js = ensure_file_vnode(
        db,
        store,
        parent_id,
        parent.as_ref(),
        THEME_JS_NAME,
        THEME_JS,
    )
    .await?
    .0;

    preferences::save_preferences(
        db,
        WebsitePreferences {
            id: 1,
            created_at: None,
            updated_at: None,
            custom_theme_css_vnode_id: Some(css.id),
            custom_theme_js_vnode_id: Some(js.id),
        },
    )
    .await?;

    tracing::info!(
        css_vnode_id = css.id,
        js_vnode_id = js.id,
        "resummarized website: custom theme preferences ready"
    );
    Ok(())
}

fn first_hardcoded_media_url(source: &str) -> Option<&str> {
    const PREFIX: &str = "/media/";
    let mut from = 0;
    while let Some(rel) = source[from..].find(PREFIX) {
        let start = from + rel;
        let rest = &source[start + PREFIX.len()..];
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 {
            let mut end = start + PREFIX.len() + digits;
            if source.as_bytes().get(end) == Some(&b'/') {
                end += 1;
            }
            return Some(&source[start..end]);
        }
        from = start + PREFIX.len();
    }
    None
}

fn has_legacy_static_url(source: &str) -> bool {
    source.contains("\"/static/") || source.contains("'/static/")
}

fn reject_nonportable_asset_urls(label: &str, source: &str) -> anyhow::Result<()> {
    if let Some(url) = first_hardcoded_media_url(source) {
        anyhow::bail!(
            "{label} must use media_url(\"/website/static/{{filename}}\"), not hardcoded vnode URL {url}"
        );
    }
    if has_legacy_static_url(source) {
        anyhow::bail!("{label} must use media_url(\"/website/static/{{filename}}\")");
    }
    Ok(())
}

/// Resolve `media_url(...)` (and other template funcs) against the seeded VNode tree.
fn render_seeded_template(db: &DatabaseConnection, source: &str) -> anyhow::Result<String> {
    reject_nonportable_asset_urls("theme css", source)?;
    let mut env = minijinja::Environment::new();
    template_funcs::register_funcs(&mut env, db.clone(), "/".into(), vec![]);
    Ok(env.render_str(source, ())?)
}

async fn ensure_page_vnode(
    db: &DatabaseConnection,
    store: &DynFilestore,
    name: &str,
    html: &[u8],
) -> anyhow::Result<(lariv_rs::plugins::filesystem::entities::VNode, bool)> {
    let segments = ["website".into(), "pages".into()];
    let parent_id = node::ensure_directory_path(db, store, None, &segments)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let parent = match parent_id {
        Some(id) => match node::get_by_id(db, id).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "get node by id for website page parent");
                None
            }
        },
        None => None,
    };

    ensure_file_vnode(db, store, parent_id, parent.as_ref(), name, html).await
}

/// Seeds blobs under `website/static/` for `media_url("/website/static/{name}")`.
async fn ensure_static_assets(db: &DatabaseConnection, store: &DynFilestore) -> anyhow::Result<()> {
    let segments = ["website".into(), "static".into()];
    let parent_id = node::ensure_directory_path(db, store, None, &segments)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let parent = match parent_id {
        Some(id) => match node::get_by_id(db, id).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "get node by id for website static parent");
                None
            }
        },
        None => None,
    };

    for asset in STATIC_ASSETS {
        let vnode = ensure_file_vnode(
            db,
            store,
            parent_id,
            parent.as_ref(),
            asset.name,
            asset.bytes,
        )
        .await?
        .0;
        tracing::info!(
            name = asset.name,
            vnode_id = vnode.id,
            media_url = %public_asset_url(vnode.id),
            bytes = asset.bytes.len(),
            "resummarized website: static asset ready"
        );
    }
    Ok(())
}

/// Drop leftover `/static/{name}` aliases from earlier seeds.
async fn remove_legacy_static_routes(db: &DatabaseConnection) -> anyhow::Result<()> {
    for asset in STATIC_ASSETS {
        let path = format!("/static/{}", asset.name);
        let res = DbRouteEntity::delete_many()
            .filter(DbRouteColumn::Path.eq(path.clone()))
            .exec(db)
            .await?;
        if res.rows_affected > 0 {
            tracing::info!(path, "resummarized website: removed legacy static route");
        }
    }
    Ok(())
}

async fn ensure_file_vnode(
    db: &DatabaseConnection,
    store: &DynFilestore,
    parent_id: Option<i64>,
    parent: Option<&lariv_rs::plugins::filesystem::entities::VNode>,
    name: &str,
    bytes: &[u8],
) -> anyhow::Result<(lariv_rs::plugins::filesystem::entities::VNode, bool)> {
    if let Some(existing) = node::find_child(db, parent_id, name, false).await? {
        if vnode_bytes_match(store, &existing, bytes).await? {
            return Ok((existing, false));
        }
        tracing::warn!(
            name,
            vnode_id = existing.id,
            stored_path = existing.file_path.as_deref().unwrap_or(""),
            "resummarized website: rewriting vnode blob"
        );
        let updated = render::replace_vnode_content(db, store, existing, bytes)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        return Ok((updated, true));
    }

    tracing::info!(name, "resummarized website: creating vnode");
    let created = node::create(
        db,
        store,
        name.into(),
        false,
        Some(NodeFile::Bytes {
            filename: name.into(),
            data: bytes.to_vec(),
        }),
        parent,
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok((created, true))
}

async fn vnode_bytes_match(
    store: &DynFilestore,
    existing: &lariv_rs::plugins::filesystem::entities::VNode,
    bytes: &[u8],
) -> anyhow::Result<bool> {
    let path = existing.file_path.as_deref().unwrap_or("");
    let mut download = match store.open(path, &existing.name).await {
        Ok(d) => d,
        Err(e) if e.is_missing() => {
            tracing::warn!(
                name = %existing.name,
                vnode_id = existing.id,
                stored_path = path,
                "resummarized website: blob missing from store"
            );
            return Ok(false);
        }
        Err(e) => return Err(anyhow::anyhow!("{e}")),
    };
    let mut current = Vec::new();
    download.reader.read_to_end(&mut current).await?;
    Ok(current == bytes)
}

async fn ensure_db_route(
    db: &DatabaseConnection,
    path: &str,
    page_id: i64,
    theme: &str,
    reset_grapes_project: bool,
) -> anyhow::Result<()> {
    if let Some(existing) = DbRouteEntity::find()
        .filter(DbRouteColumn::Path.eq(path))
        .one(db)
        .await?
    {
        let mut am: db_route::ActiveModel = existing.into();
        am.page_id = Set(page_id);
        am.is_active = Set(true);
        am.theme = Set(theme.into());
        if reset_grapes_project {
            // Drop stale GrapesJS project JSON so the builder reloads from seeded HTML.
            am.grapes_project = Set(None);
        }
        am.updated_at = Set(Some(Utc::now()));
        am.update(db).await?;
        tracing::info!(path, page_id, "resummarized website: updated db route");
        return Ok(());
    }

    let now = Utc::now();
    db_route::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        path: Set(path.into()),
        page_id: Set(page_id),
        is_active: Set(true),
        theme: Set(theme.into()),
        grapes_project: Set(None),
    }
    .insert(db)
    .await?;
    tracing::info!(path, page_id, "resummarized website: created db route");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        HOMEPAGE_HTML, SUBSCRIBE_HTML, THEME_CSS, first_hardcoded_media_url, has_legacy_static_url,
        reject_nonportable_asset_urls,
    };

    #[test]
    fn seeded_assets_use_media_url_not_static_routes() {
        let css = std::str::from_utf8(THEME_CSS).expect("theme css is utf-8");
        for (label, source) in [
            ("homepage.html", HOMEPAGE_HTML),
            ("subscribe.html", SUBSCRIBE_HTML),
            ("resummarized.css", css),
        ] {
            assert_eq!(
                first_hardcoded_media_url(source),
                None,
                "{label} hardcodes a /media/{{id}}/ vnode URL"
            );
            assert!(
                !has_legacy_static_url(source),
                "{label} still references /static/ instead of media_url"
            );
            assert!(
                source.contains("media_url('/website/static/"),
                "{label} should call media_url(\"/website/static/...\")"
            );
        }
    }

    #[test]
    fn reject_nonportable_asset_urls_catches_legacy_refs() {
        let err = reject_nonportable_asset_urls("page", r#"<img src="/media/23/">"#).unwrap_err();
        assert!(err.to_string().contains("/media/23/"), "{err}");
        let err =
            reject_nonportable_asset_urls("page", r#"<img src="/static/logo.svg">"#).unwrap_err();
        assert!(err.to_string().contains("media_url"), "{err}");
    }
}
