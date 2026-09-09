use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use tracing::{info, warn};

use super::{
    description::{parse_bse_datetime, parse_bse_description},
    entities::{
        item::{self, ActiveModel as ItemAM, Entity as ItemEntity, Model as ItemModel},
        status::{ActiveModel as StatusAM, Entity as StatusEntity},
    },
    feeds::{BseFeedKind, normalize_item_link, scripcode_from_title},
    rss::{content_hash, parse_rss},
    state::BseState,
};

const CHROME_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
const BSE_HOME: &str = "https://www.bseindia.com/";
const BSE_RSS_PAGE: &str = "https://beta.bseindia.com/rss-feed.html";
const INTER_FEED_DELAY: Duration = Duration::from_millis(500);

pub fn build_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .cookie_store(true)
        .user_agent(CHROME_UA)
        .gzip(true)
        .brotli(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(60))
        .build()?)
}

pub async fn warmup(client: &reqwest::Client) -> anyhow::Result<()> {
    let response = client
        .get(BSE_HOME)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await?;
    if !response.status().is_success() {
        anyhow::bail!("BSE homepage warmup HTTP {}", response.status());
    }
    let _ = response.bytes().await?;
    Ok(())
}

pub async fn fetch_all(state: &BseState) -> anyhow::Result<()> {
    if let Err(e) = warmup(&state.client).await {
        warn!(error = %e, "BSE cookie warmup failed");
        return Err(e);
    }
    for kind in BseFeedKind::ALL {
        if let Err(e) = fetch_feed_warmed(state, *kind).await {
            warn!(feed = kind.slug(), error = %e, "BSE feed fetch failed");
        }
        tokio::time::sleep(INTER_FEED_DELAY).await;
    }
    Ok(())
}

pub async fn fetch_feed(state: &BseState, kind: BseFeedKind) -> anyhow::Result<usize> {
    warmup(&state.client).await?;
    fetch_feed_warmed(state, kind).await
}

async fn fetch_feed_warmed(state: &BseState, kind: BseFeedKind) -> anyhow::Result<usize> {
    match fetch_and_upsert(state, kind).await {
        Ok((inserted, last_build)) => {
            record_status(state, kind, last_build, None).await;
            info!(feed = kind.slug(), inserted, "BSE feed fetched");
            Ok(inserted)
        }
        Err(e) => {
            record_status(state, kind, None, Some(e.to_string())).await;
            Err(e)
        }
    }
}

async fn fetch_and_upsert(
    state: &BseState,
    kind: BseFeedKind,
) -> anyhow::Result<(usize, Option<chrono::DateTime<chrono::Utc>>)> {
    let response = state
        .client
        .get(kind.url())
        .header(
            "Accept",
            "application/rss+xml, application/xml, text/xml, */*",
        )
        .header("Referer", BSE_RSS_PAGE)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await?;
    let status = response.status();
    let bytes = response.bytes().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}");
    }
    let xml = String::from_utf8_lossy(&bytes);
    let doc = parse_rss(&xml).map_err(|e| anyhow::anyhow!("parse {}: {e}", kind.slug()))?;
    let last_build = doc
        .channel
        .last_build_date
        .as_deref()
        .and_then(parse_bse_datetime);
    let now = Utc::now();

    let mut existing_by_hash: HashMap<String, ItemModel> = ItemEntity::find()
        .filter(item::Column::FeedKind.eq(kind.slug()))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|row| (row.content_hash.clone(), row))
        .collect();

    let mut inserted = 0usize;
    for entry in doc.channel.items {
        let link = normalize_item_link(&entry.link);
        let scripcode = entry
            .scripcode
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| scripcode_from_title(&entry.title));
        let scripcode_hash = scripcode.as_deref().unwrap_or("");
        let hash = content_hash(
            kind.slug(),
            &entry.title,
            &link,
            &entry.description,
            &entry.pub_date,
            scripcode_hash,
        );
        let parsed = parse_bse_description(&entry.description);
        let pub_date = parsed.inferred_pub_date(&entry.pub_date);
        if let Some(row) = existing_by_hash.get_mut(&hash) {
            if row.pub_date.is_none() && pub_date.is_some() {
                let mut am: ItemAM = row.clone().into();
                am.pub_date = Set(pub_date);
                am.updated_at = Set(Some(now));
                am.update(&state.db).await?;
                row.pub_date = pub_date;
            }
            continue;
        }
        let mut am = ItemAM {
            feed_kind: Set(kind.slug().to_string()),
            title: Set(entry.title),
            link: Set(link),
            description: Set(parsed.remainder.clone()),
            pub_date: Set(pub_date),
            content_hash: Set(hash),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        };
        parsed.apply_extracted(&mut am);
        if let Some(code) = scripcode {
            am.scripcode = Set(Some(code));
        }
        ItemEntity::insert(am)
            .on_conflict(
                OnConflict::column(item::Column::ContentHash)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&state.db)
            .await?;
        inserted += 1;
    }

    Ok((inserted, last_build))
}

async fn record_status(
    state: &BseState,
    kind: BseFeedKind,
    last_build_date: Option<chrono::DateTime<chrono::Utc>>,
    last_error: Option<String>,
) {
    let now = Utc::now();
    let slug = kind.slug().to_string();
    let existing = StatusEntity::find_by_id(slug.clone())
        .one(&state.db)
        .await
        .ok()
        .flatten();
    let result = if let Some(row) = existing {
        let mut am: StatusAM = row.into();
        am.updated_at = Set(Some(now));
        am.last_fetched_at = Set(Some(now));
        if last_build_date.is_some() {
            am.last_build_date = Set(last_build_date);
        }
        am.last_error = Set(last_error);
        am.update(&state.db).await.map(|_| ())
    } else {
        StatusAM {
            feed_kind: Set(slug),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            last_build_date: Set(last_build_date),
            last_error: Set(last_error),
            last_fetched_at: Set(Some(now)),
        }
        .insert(&state.db)
        .await
        .map(|_| ())
    };
    if let Err(e) = result {
        warn!(error = %e, feed = kind.slug(), "failed to write BSE feed status");
    }
}
