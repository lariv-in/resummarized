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
    feeds::EuronextFeedKind,
    rss::{
        ParsedItems, content_hash, looks_complete, looks_like_html, parse_euronext_datetime,
        parse_rss,
    },
    state::EuronextState,
};

/// Browser UA — some automated clients have seen HTTP 422 from Athens.
const ATHENS_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const INTER_FEED_DELAY: Duration = Duration::from_millis(500);
const RSS_DOWNLOAD_ATTEMPTS: u8 = 3;

fn display_error(err: &impl std::fmt::Display) -> String {
    format!("{err:#}")
}

fn opt_text(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

pub fn build_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(ATHENS_UA)
        .gzip(true)
        .brotli(true)
        .use_native_tls()
        .connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(60))
        .build()?)
}

pub async fn fetch_all(state: &EuronextState) -> anyhow::Result<()> {
    for kind in EuronextFeedKind::ALL {
        if let Err(e) = fetch_one(state, *kind).await {
            warn!(feed = kind.slug(), error = %e, "Euronext feed fetch failed");
        }
        tokio::time::sleep(INTER_FEED_DELAY).await;
    }
    Ok(())
}

pub async fn fetch_feed(state: &EuronextState, kind: EuronextFeedKind) -> anyhow::Result<usize> {
    fetch_one(state, kind).await
}

async fn fetch_one(state: &EuronextState, kind: EuronextFeedKind) -> anyhow::Result<usize> {
    match fetch_and_upsert(state, kind).await {
        Ok((inserted, last_build)) => {
            record_status(state, kind, last_build, None).await;
            info!(feed = kind.slug(), inserted, "Euronext feed fetched");
            Ok(inserted)
        }
        Err(e) => {
            record_status(state, kind, None, Some(e.to_string())).await;
            Err(e)
        }
    }
}

async fn download_body(state: &EuronextState, url: &str) -> anyhow::Result<String> {
    let response = state
        .client
        .get(url)
        .header("User-Agent", ATHENS_UA)
        .header(
            "Accept",
            "application/rss+xml, application/xml, text/xml, */*",
        )
        .header("Referer", "https://athens.euronext.com/en/rss")
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

async fn download_rss(state: &EuronextState, kind: EuronextFeedKind) -> anyhow::Result<String> {
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
                    "Euronext feed returned HTML"
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
                    "truncated Euronext feed"
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
                    "Euronext feed download failed"
                );
                last_err = Some(e);
            }
        }
        if attempt < RSS_DOWNLOAD_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(400 * u64::from(attempt))).await;
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Euronext feed download failed")))
}

async fn fetch_and_upsert(
    state: &EuronextState,
    kind: EuronextFeedKind,
) -> anyhow::Result<(usize, Option<chrono::DateTime<chrono::Utc>>)> {
    let xml = download_rss(state, kind).await?;
    let doc = parse_document(&xml).map_err(|e| anyhow::anyhow!("parse {}: {e}", kind.slug()))?;
    let last_build = doc
        .last_build_date
        .as_deref()
        .and_then(parse_euronext_datetime);
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
        let guid = opt_text(entry.guid.as_ref().map(|g| g.as_str()));
        let company_name = entry
            .company
            .as_ref()
            .and_then(|c| opt_text(c.company_name.as_deref()));
        let company_ticker = entry
            .company
            .as_ref()
            .and_then(|c| opt_text(c.company_ticker.as_deref()));
        let attachment = opt_text(entry.attachment.as_deref());
        let attachment_title = opt_text(entry.attachment_title.as_deref());
        let guid_hash = guid.as_deref().unwrap_or("");
        let hash = content_hash(
            kind.slug(),
            &entry.title,
            &entry.link,
            &entry.description,
            &entry.pub_date,
            guid_hash,
        );
        let pub_date = parse_euronext_datetime(&entry.pub_date);
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
            company_name: Set(company_name),
            company_ticker: Set(company_ticker),
            attachment: Set(attachment),
            attachment_title: Set(attachment_title),
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
    state: &EuronextState,
    kind: EuronextFeedKind,
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
        warn!(error = %e, feed = kind.slug(), "failed to write Euronext feed status");
    }
}

#[cfg(test)]
mod tests {
    use super::build_client;

    #[test]
    fn build_client_constructs() {
        build_client().expect("euronext http client");
    }

    #[tokio::test]
    #[ignore = "hits athens.euronext.com"]
    async fn live_issuer_announcements_is_rss() {
        let client = build_client().expect("client");
        let url = crate::euronext::feeds::EuronextFeedKind::IssuerAnnouncements.url();
        let response = client
            .get(url)
            .header("User-Agent", super::ATHENS_UA)
            .header(
                "Accept",
                "application/rss+xml, application/xml, text/xml, */*",
            )
            .header("Referer", "https://athens.euronext.com/en/rss")
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
        let doc = crate::euronext::rss::parse_rss(&xml).expect("parse live issuer rss");
        assert!(!doc.channel.items.is_empty());
        assert!(
            doc.channel
                .items
                .iter()
                .any(|item| item.company.is_some() || item.attachment.is_some()),
            "expected at least one item with company or attachment"
        );
    }
}
