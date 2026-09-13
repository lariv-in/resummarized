use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use sha2::{Digest, Sha256};
use tracing::{info, warn};

use super::{
    atom,
    entities::{
        item::{self, ActiveModel as ItemAM, Entity as ItemEntity, Model as ItemModel},
        status::{ActiveModel as StatusAM, Entity as StatusEntity},
    },
    feeds::{EdgarFeedKind, form_type_from_title},
    state::EdgarState,
};

pub const SEC_UA: &str = "Resummarized/0.1 (admin@example.com)";
const INTER_FEED_DELAY: Duration = Duration::from_millis(500);
const ATOM_DOWNLOAD_ATTEMPTS: u8 = 3;

fn display_error(err: &impl std::fmt::Display) -> String {
    format!("{err:#}")
}

pub fn build_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(SEC_UA)
        .gzip(true)
        .brotli(true)
        .use_native_tls()
        .connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(60))
        .build()?)
}

pub fn parse_edgar_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(s)
        .or_else(|_| DateTime::parse_from_rfc2822(s))
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub fn content_hash(
    feed_kind: &str,
    title: &str,
    link: &str,
    description: &str,
    pub_date: &str,
    guid: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(feed_kind.as_bytes());
    hasher.update([0u8]);
    hasher.update(title.as_bytes());
    hasher.update([0u8]);
    hasher.update(link.as_bytes());
    hasher.update([0u8]);
    hasher.update(description.as_bytes());
    hasher.update([0u8]);
    hasher.update(pub_date.as_bytes());
    hasher.update([0u8]);
    hasher.update(guid.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn fetch_all(state: &EdgarState) -> anyhow::Result<()> {
    for kind in EdgarFeedKind::ALL {
        if let Err(e) = fetch_one(state, *kind).await {
            warn!(feed = kind.slug(), error = %e, "EDGAR feed fetch failed");
        }
        tokio::time::sleep(INTER_FEED_DELAY).await;
    }
    Ok(())
}

pub async fn fetch_feed(state: &EdgarState, kind: EdgarFeedKind) -> anyhow::Result<usize> {
    fetch_one(state, kind).await
}

async fn fetch_one(state: &EdgarState, kind: EdgarFeedKind) -> anyhow::Result<usize> {
    match fetch_and_upsert(state, kind).await {
        Ok((inserted, last_build)) => {
            record_status(state, kind, last_build, None).await;
            info!(feed = kind.slug(), inserted, "EDGAR feed fetched");
            Ok(inserted)
        }
        Err(e) => {
            record_status(state, kind, None, Some(e.to_string())).await;
            Err(e)
        }
    }
}

async fn download_body(state: &EdgarState, url: &str) -> anyhow::Result<String> {
    let response = state
        .client
        .get(url)
        .header("User-Agent", SEC_UA)
        .header(
            "Accept",
            "application/atom+xml, application/xml, text/xml, */*",
        )
        .send()
        .await?;
    let status = response.status();
    let bytes = response.bytes().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}");
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn looks_complete_atom(xml: &str) -> bool {
    xml.trim_end().to_ascii_lowercase().ends_with("</feed>")
}

fn looks_like_html(xml: &str) -> bool {
    let t = xml.trim_start_matches('\u{feff}').trim_start();
    let head = t.get(..256).unwrap_or(t).to_ascii_lowercase();
    head.starts_with("<!doctype html") || head.contains("<html")
}

async fn download_atom(state: &EdgarState, kind: EdgarFeedKind) -> anyhow::Result<String> {
    let url = kind.url();
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=ATOM_DOWNLOAD_ATTEMPTS {
        match download_body(state, url).await {
            Ok(xml) if looks_like_html(&xml) => {
                last_err = Some(anyhow::anyhow!(
                    "HTML instead of feed ({} bytes)",
                    xml.len()
                ));
                warn!(
                    feed = kind.slug(),
                    attempt,
                    url,
                    bytes = xml.len(),
                    "EDGAR feed returned HTML"
                );
            }
            Ok(xml) if looks_complete_atom(&xml) => {
                return Ok(xml);
            }
            Ok(xml) => {
                last_err = Some(anyhow::anyhow!("truncated feed ({} bytes)", xml.len()));
                warn!(
                    feed = kind.slug(),
                    attempt,
                    url,
                    bytes = xml.len(),
                    "truncated EDGAR feed"
                );
                if attempt == ATOM_DOWNLOAD_ATTEMPTS {
                    return Ok(xml);
                }
            }
            Err(e) => {
                warn!(
                    feed = kind.slug(),
                    attempt,
                    url,
                    error = %display_error(&e),
                    "EDGAR feed download failed"
                );
                last_err = Some(e);
            }
        }
        if attempt < ATOM_DOWNLOAD_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(400 * u64::from(attempt))).await;
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("EDGAR feed download failed")))
}

async fn fetch_and_upsert(
    state: &EdgarState,
    kind: EdgarFeedKind,
) -> anyhow::Result<(usize, Option<DateTime<Utc>>)> {
    let xml = download_atom(state, kind).await?;
    let doc = atom::parse_atom(&xml).map_err(|e| anyhow::anyhow!("parse {}: {e}", kind.slug()))?;
    let last_build = doc
        .last_build_date
        .as_deref()
        .and_then(parse_edgar_datetime);
    let now = Utc::now();

    let mut existing_by_hash: HashMap<String, ItemModel> = ItemEntity::find()
        .filter(item::Column::FeedKind.eq(kind.slug()))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|row| (row.content_hash.clone(), row))
        .collect();

    let mut inserted = 0usize;
    for entry in doc.items {
        let guid = entry.guid.filter(|s| !s.is_empty());
        let author = entry
            .author
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let category = entry
            .category
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| {
                kind.stores_form_category()
                    .then(|| form_type_from_title(&entry.title))
                    .flatten()
            });
        let guid_hash = guid.as_deref().unwrap_or("");
        let hash = content_hash(
            kind.slug(),
            &entry.title,
            &entry.link,
            &entry.description,
            &entry.pub_date,
            guid_hash,
        );
        let pub_date = parse_edgar_datetime(&entry.pub_date);
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
        let am = ItemAM {
            feed_kind: Set(kind.slug().to_string()),
            title: Set(entry.title),
            link: Set(entry.link),
            description: Set(entry.description),
            pub_date: Set(pub_date),
            guid: Set(guid),
            category: Set(category),
            author: Set(author),
            content_hash: Set(hash),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        };
        let insert = ItemEntity::insert(am)
            .on_conflict(
                OnConflict::column(item::Column::ContentHash)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&state.db)
            .await?;
        if insert.last_insert_id != 0 {
            inserted += 1;
        }
    }

    Ok((inserted, last_build))
}

async fn record_status(
    state: &EdgarState,
    kind: EdgarFeedKind,
    last_build_date: Option<DateTime<Utc>>,
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
        warn!(error = %e, feed = kind.slug(), "failed to write EDGAR feed status");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_client_constructs() {
        build_client().expect("edgar http client");
    }

    #[test]
    fn parse_edgar_datetime_formats() {
        let parsed = parse_edgar_datetime("2026-09-10T16:12:55-04:00").expect("rfc3339");
        assert_eq!(parsed.to_rfc3339(), "2026-09-10T20:12:55+00:00");
    }

    #[test]
    fn content_hash_is_deterministic() {
        let h1 = content_hash("8-k", "Title", "https://sec.gov", "Desc", "2026-09-10", "guid1");
        let h2 = content_hash("8-k", "Title", "https://sec.gov", "Desc", "2026-09-10", "guid1");
        assert_eq!(h1, h2);
    }
}
