use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use tracing::{info, warn};

use super::{
    entities::{
        item::{self, ActiveModel as ItemAM, Entity as ItemEntity, Model as ItemModel},
        status::{ActiveModel as StatusAM, Entity as StatusEntity},
    },
    feeds::JpxFeedKind,
    rss::{
        ParsedItems, content_hash, looks_complete, looks_like_html, parse_jpx_datetime, parse_rss,
    },
    state::JpxState,
};

/// Browser UA — some automated clients have seen HTTP 500 from jpx.co.jp.
const JPX_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const INTER_FEED_DELAY: Duration = Duration::from_millis(500);
const RSS_DOWNLOAD_ATTEMPTS: u8 = 3;
const REFERER: &str = "https://www.jpx.co.jp/english/rss/index.html";

fn display_error(err: &impl std::fmt::Display) -> String {
    format!("{err:#}")
}

fn opt_text(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn trim_owned(s: String) -> String {
    s.trim().to_string()
}

pub fn build_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(JPX_UA)
        .gzip(true)
        .brotli(true)
        .use_native_tls()
        .connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(60))
        .build()?)
}

pub async fn fetch_all(state: &JpxState) -> anyhow::Result<()> {
    for kind in JpxFeedKind::ALL {
        if let Err(e) = fetch_one(state, *kind).await {
            warn!(feed = kind.slug(), error = %e, "JPX feed fetch failed");
        }
        tokio::time::sleep(INTER_FEED_DELAY).await;
    }
    Ok(())
}

pub async fn fetch_feed(state: &JpxState, kind: JpxFeedKind) -> anyhow::Result<usize> {
    fetch_one(state, kind).await
}

async fn fetch_one(state: &JpxState, kind: JpxFeedKind) -> anyhow::Result<usize> {
    match fetch_and_upsert(state, kind).await {
        Ok((inserted, last_build)) => {
            record_status(state, kind, last_build, None).await;
            info!(feed = kind.slug(), inserted, "JPX feed fetched");
            Ok(inserted)
        }
        Err(e) => {
            record_status(state, kind, None, Some(e.to_string())).await;
            Err(e)
        }
    }
}

async fn download_body(state: &JpxState, url: &str) -> anyhow::Result<String> {
    let response = state
        .client
        .get(url)
        .header("User-Agent", JPX_UA)
        .header(
            "Accept",
            "application/rss+xml, application/xml, text/xml, */*",
        )
        .header("Referer", REFERER)
        .send()
        .await?;
    let status = response.status();
    let bytes = response.bytes().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}");
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn parse_document(xml: &str) -> Result<ParsedItems, quick_xml::DeError> {
    parse_rss(xml).map(ParsedItems::from_rss)
}

async fn download_rss(state: &JpxState, kind: JpxFeedKind) -> anyhow::Result<String> {
    let url = kind.url();
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=RSS_DOWNLOAD_ATTEMPTS {
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
                    "JPX feed returned HTML"
                );
            }
            Ok(xml) if looks_complete(&xml) => {
                return Ok(xml);
            }
            Ok(xml) => {
                last_err = Some(anyhow::anyhow!("truncated feed ({} bytes)", xml.len()));
                warn!(
                    feed = kind.slug(),
                    attempt,
                    url,
                    bytes = xml.len(),
                    "truncated JPX feed"
                );
                if attempt == RSS_DOWNLOAD_ATTEMPTS {
                    return Ok(xml);
                }
            }
            Err(e) => {
                warn!(
                    feed = kind.slug(),
                    attempt,
                    url,
                    error = %display_error(&e),
                    "JPX feed download failed"
                );
                last_err = Some(e);
            }
        }
        if attempt < RSS_DOWNLOAD_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(400 * u64::from(attempt))).await;
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("JPX feed download failed")))
}

async fn fetch_and_upsert(
    state: &JpxState,
    kind: JpxFeedKind,
) -> anyhow::Result<(usize, Option<chrono::DateTime<chrono::Utc>>)> {
    let xml = download_rss(state, kind).await?;
    let doc = parse_document(&xml).map_err(|e| anyhow::anyhow!("parse {}: {e}", kind.slug()))?;
    let last_build = doc.last_build_date.as_deref().and_then(parse_jpx_datetime);
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
        let title = trim_owned(entry.title);
        let link = trim_owned(entry.link);
        let description = trim_owned(entry.description);
        let pub_date_raw = trim_owned(entry.pub_date);
        let guid = opt_text(entry.guid.as_ref().map(|g| g.as_str()));
        let guid_hash = guid.as_deref().unwrap_or("");
        let hash = content_hash(
            kind.slug(),
            &title,
            &link,
            &description,
            &pub_date_raw,
            guid_hash,
        );
        let pub_date = parse_jpx_datetime(&pub_date_raw);
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
            title: Set(title),
            link: Set(link),
            description: Set(description),
            pub_date: Set(pub_date),
            guid: Set(guid),
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
    state: &JpxState,
    kind: JpxFeedKind,
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
        warn!(error = %e, feed = kind.slug(), "failed to write JPX feed status");
    }
}

#[cfg(test)]
mod tests {
    use super::build_client;

    #[test]
    fn build_client_constructs() {
        build_client().expect("jpx http client");
    }

    #[tokio::test]
    #[ignore = "hits www.jpx.co.jp"]
    async fn live_market_news_is_rss() {
        let client = build_client().expect("client");
        let url = crate::jpx::feeds::JpxFeedKind::MarketNews.url();
        let response = client
            .get(url)
            .header("User-Agent", super::JPX_UA)
            .header(
                "Accept",
                "application/rss+xml, application/xml, text/xml, */*",
            )
            .header("Referer", super::REFERER)
            .send()
            .await
            .expect("send");
        let status = response.status();
        let xml = response.text().await.expect("body");
        assert!(
            status.is_success() && xml.contains("<rss") && xml.contains("<item>"),
            "status={status} body={}",
            &xml[..xml.len().min(300)]
        );
        let doc = crate::jpx::rss::parse_rss(&xml).expect("parse live market news rss");
        assert!(!doc.channel.items.is_empty());
        assert!(
            doc.channel
                .items
                .iter()
                .any(|item| item.guid.is_some() && !item.title.trim().is_empty()),
            "expected at least one item with guid and title"
        );
    }
}
