use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use tracing::{info, warn};

use super::{
    brsr::{self, parse_brsr_xbrl},
    description::{parse_nse_datetime, parse_nse_description},
    entities::{
        item::{self, ActiveModel as ItemAM, Entity as ItemEntity, Model as ItemModel},
        status::{ActiveModel as StatusAM, Entity as StatusEntity},
    },
    extras::{self, LinkedFacts},
    feeds::NseFeedKind,
    fr::{self, parse_fr_xbrl},
    ic::{self, parse_ic_xbrl},
    iff::{self, parse_iff_xbrl},
    it::{self, parse_it_xbrl},
    rpt::{self, parse_rpt_xbrl},
    rss::{content_hash, looks_complete, looks_like_html, parse_rss},
    scr::{self, parse_scr_xbrl},
    shp::{self, parse_shp_xbrl},
    sod::{self, parse_sod_xbrl},
    state::NseState,
    uhp::{self, parse_uhp_xbrl},
    voting::{self, parse_voting_xbrl},
};

const CHROME_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
const NSE_HOME: &str = "https://www.nseindia.com/";
const NSE_RSS_PAGE: &str = "https://www.nseindia.com/static/rss-feed";
const INTER_FEED_DELAY: Duration = Duration::from_millis(500);
const INTER_XML_DELAY: Duration = Duration::from_millis(200);
const RSS_DOWNLOAD_ATTEMPTS: u8 = 3;

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
        .get(NSE_HOME)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await?;
    if !response.status().is_success() {
        anyhow::bail!("NSE homepage warmup HTTP {}", response.status());
    }
    let _ = response.bytes().await?;
    Ok(())
}

pub async fn fetch_all(state: &NseState) -> anyhow::Result<()> {
    if let Err(e) = warmup(&state.client).await {
        warn!(error = %e, "NSE cookie warmup failed");
        return Err(e);
    }
    for kind in NseFeedKind::ALL {
        if let Err(e) = fetch_feed_warmed(state, *kind).await {
            warn!(feed = kind.slug(), error = %e, "NSE feed fetch failed");
        }
        tokio::time::sleep(INTER_FEED_DELAY).await;
    }
    Ok(())
}

pub async fn fetch_feed(state: &NseState, kind: NseFeedKind) -> anyhow::Result<usize> {
    warmup(&state.client).await?;
    fetch_feed_warmed(state, kind).await
}

async fn fetch_feed_warmed(state: &NseState, kind: NseFeedKind) -> anyhow::Result<usize> {
    match fetch_and_upsert(state, kind).await {
        Ok((inserted, last_build)) => {
            record_status(state, kind, last_build, None).await;
            info!(feed = kind.slug(), inserted, "NSE feed fetched");
            Ok(inserted)
        }
        Err(e) => {
            record_status(state, kind, None, Some(e.to_string())).await;
            Err(e)
        }
    }
}

async fn download_rss_once(state: &NseState, kind: NseFeedKind) -> anyhow::Result<String> {
    let response = state
        .client
        .get(kind.url())
        .header(
            "Accept",
            "application/rss+xml, application/xml, text/xml, */*",
        )
        .header("Referer", NSE_RSS_PAGE)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await?;
    let status = response.status();
    let bytes = response.bytes().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}");
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

async fn download_rss(state: &NseState, kind: NseFeedKind) -> anyhow::Result<String> {
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=RSS_DOWNLOAD_ATTEMPTS {
        match download_rss_once(state, kind).await {
            Ok(xml) if looks_like_html(&xml) => {
                last_err = Some(anyhow::anyhow!("HTML instead of RSS ({} bytes)", xml.len()));
                warn!(
                    feed = kind.slug(),
                    attempt,
                    bytes = xml.len(),
                    "NSE RSS returned HTML, retrying"
                );
            }
            Ok(xml) if looks_complete(&xml) => return Ok(xml),
            Ok(xml) => {
                last_err = Some(anyhow::anyhow!(
                    "truncated RSS ({} bytes, missing </rss>)",
                    xml.len()
                ));
                warn!(
                    feed = kind.slug(),
                    attempt,
                    bytes = xml.len(),
                    "truncated NSE RSS, retrying"
                );
                if attempt == RSS_DOWNLOAD_ATTEMPTS {
                    return Ok(xml);
                }
            }
            Err(e) => {
                warn!(
                    feed = kind.slug(),
                    attempt,
                    error = %e,
                    "NSE RSS download failed, retrying"
                );
                last_err = Some(e);
            }
        }
        if attempt < RSS_DOWNLOAD_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(400 * u64::from(attempt))).await;
            if let Err(e) = warmup(&state.client).await {
                warn!(error = %e, "NSE re-warmup failed");
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("NSE RSS download failed")))
}

async fn fetch_and_upsert(
    state: &NseState,
    kind: NseFeedKind,
) -> anyhow::Result<(usize, Option<chrono::DateTime<chrono::Utc>>)> {
    let xml = download_rss(state, kind).await?;
    let doc = parse_rss(&xml).map_err(|e| anyhow::anyhow!("parse {}: {e}", kind.filename()))?;
    let last_build = doc
        .channel
        .last_build_date
        .as_deref()
        .and_then(parse_nse_datetime);
    let now = Utc::now();

    let existing_rows = ItemEntity::find()
        .filter(item::Column::FeedKind.eq(kind.slug()))
        .all(&state.db)
        .await?;
    let existing_ids: Vec<i64> = existing_rows.iter().map(|row| row.id).collect();
    let extras = extras::load_map(&state.db, kind, &existing_ids).await?;
    for row in &existing_rows {
        let parsed = parse_nse_description(&row.description);
        let extra = extras.get(&row.id);
        let xml_facts = if extras::needs_linked_facts(kind, extra) {
            fetch_linked_for_kind(&state.client, kind, &row.link).await
        } else {
            None
        };
        if !parsed.any_extracted() && xml_facts.is_none() {
            continue;
        }
        if parsed.any_extracted() {
            let mut am: ItemAM = row.clone().into();
            am.description = Set(parsed.remainder.clone());
            am.updated_at = Set(Some(now));
            am.update(&state.db).await?;
        }
        extras::upsert(&state.db, kind, row.id, &parsed, xml_facts.as_ref()).await?;
    }

    let mut existing_by_hash: HashMap<String, ItemModel> = existing_rows
        .into_iter()
        .map(|row| (row.content_hash.clone(), row))
        .collect();

    let mut inserted = 0usize;
    for entry in doc.channel.items {
        let hash = content_hash(
            kind.slug(),
            &entry.title,
            &entry.link,
            &entry.description,
            &entry.pub_date,
        );
        let parsed = parse_nse_description(&entry.description);
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
        let xml_facts = fetch_linked_for_kind(&state.client, kind, &entry.link).await;
        let am = ItemAM {
            feed_kind: Set(kind.slug().to_string()),
            title: Set(entry.title),
            link: Set(entry.link),
            description: Set(parsed.remainder.clone()),
            pub_date: Set(pub_date),
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
            extras::upsert(
                &state.db,
                kind,
                insert.last_insert_id,
                &parsed,
                xml_facts.as_ref(),
            )
            .await?;
            inserted += 1;
        }
    }

    Ok((inserted, last_build))
}

async fn fetch_linked_for_kind(
    client: &reqwest::Client,
    kind: NseFeedKind,
    url: &str,
) -> Option<LinkedFacts> {
    let facts = match kind {
        NseFeedKind::Brsr => fetch_brsr_facts(client, url).await.map(LinkedFacts::Brsr),
        NseFeedKind::VotingResults => fetch_vote_facts(client, url).await.map(LinkedFacts::Vote),
        NseFeedKind::UnitholdingPatterns => {
            fetch_uhp_facts(client, url).await.map(LinkedFacts::Uhp)
        }
        NseFeedKind::StatementOfDeviation => {
            fetch_sod_facts(client, url).await.map(LinkedFacts::Sod)
        }
        NseFeedKind::ShareholdingPattern => {
            fetch_shp_facts(client, url).await.map(LinkedFacts::Shp)
        }
        NseFeedKind::SecretarialCompliance => {
            fetch_scr_facts(client, url).await.map(LinkedFacts::Scr)
        }
        NseFeedKind::RelatedPartyTransactions => {
            fetch_rpt_facts(client, url).await.map(LinkedFacts::Rpt)
        }
        NseFeedKind::InvestorComplaints => fetch_ic_facts(client, url).await.map(LinkedFacts::Ic),
        NseFeedKind::InsiderTrading => fetch_it_facts(client, url).await.map(LinkedFacts::It),
        NseFeedKind::IntegratedFilingFinancials => {
            fetch_iff_facts(client, url).await.map(LinkedFacts::Iff)
        }
        NseFeedKind::FinancialResults => fetch_fr_facts(client, url).await.map(LinkedFacts::Fr),
        _ => None,
    };
    if facts.is_some() {
        tokio::time::sleep(INTER_XML_DELAY).await;
    }
    facts
}

async fn fetch_brsr_facts(client: &reqwest::Client, url: &str) -> Option<brsr::BrsrFacts> {
    fetch_linked_xml(client, url, parse_brsr_xbrl, "BRSR").await
}

async fn fetch_vote_facts(client: &reqwest::Client, url: &str) -> Option<voting::VoteFacts> {
    fetch_linked_xml(client, url, parse_voting_xbrl, "voting results").await
}

async fn fetch_uhp_facts(client: &reqwest::Client, url: &str) -> Option<uhp::UhpFacts> {
    fetch_linked_xml(client, url, parse_uhp_xbrl, "unitholding pattern").await
}

async fn fetch_sod_facts(client: &reqwest::Client, url: &str) -> Option<sod::SodFacts> {
    fetch_linked_xml(client, url, parse_sod_xbrl, "statement of deviation").await
}

async fn fetch_shp_facts(client: &reqwest::Client, url: &str) -> Option<shp::ShpFacts> {
    fetch_linked_xml(client, url, parse_shp_xbrl, "shareholding pattern").await
}

async fn fetch_scr_facts(client: &reqwest::Client, url: &str) -> Option<scr::ScrFacts> {
    fetch_linked_xml(client, url, parse_scr_xbrl, "secretarial compliance").await
}

async fn fetch_rpt_facts(client: &reqwest::Client, url: &str) -> Option<rpt::RptFacts> {
    fetch_linked_xml(client, url, parse_rpt_xbrl, "related party transactions").await
}

async fn fetch_ic_facts(client: &reqwest::Client, url: &str) -> Option<ic::IcFacts> {
    fetch_linked_xml(client, url, parse_ic_xbrl, "investor complaints").await
}

async fn fetch_it_facts(client: &reqwest::Client, url: &str) -> Option<it::ItFacts> {
    fetch_linked_xml(client, url, parse_it_xbrl, "insider trading").await
}

async fn fetch_iff_facts(client: &reqwest::Client, url: &str) -> Option<iff::IffFacts> {
    fetch_linked_xml(client, url, parse_iff_xbrl, "integrated filing financials").await
}

async fn fetch_fr_facts(client: &reqwest::Client, url: &str) -> Option<fr::FrFacts> {
    fetch_linked_xml(client, url, parse_fr_xbrl, "financial results").await
}

async fn fetch_linked_xml<T>(
    client: &reqwest::Client,
    url: &str,
    parse: fn(&str) -> T,
    label: &str,
) -> Option<T>
where
    T: HasFacts,
{
    if !url.to_ascii_lowercase().ends_with(".xml") {
        return None;
    }
    match get_xml(client, url).await {
        Ok(xml) => {
            let facts = parse(&xml);
            if facts.has_identity() {
                Some(facts)
            } else {
                warn!(url, label, "XBRL parsed with no identity facts");
                None
            }
        }
        Err(e) => {
            warn!(url, label, error = %e, "XBRL fetch failed");
            None
        }
    }
}

trait HasFacts {
    fn has_identity(&self) -> bool;
}

impl HasFacts for brsr::BrsrFacts {
    fn has_identity(&self) -> bool {
        brsr::BrsrFacts::any(self)
    }
}

impl HasFacts for voting::VoteFacts {
    fn has_identity(&self) -> bool {
        voting::VoteFacts::any(self)
    }
}

impl HasFacts for uhp::UhpFacts {
    fn has_identity(&self) -> bool {
        uhp::UhpFacts::any(self)
    }
}

impl HasFacts for sod::SodFacts {
    fn has_identity(&self) -> bool {
        sod::SodFacts::any(self)
    }
}

impl HasFacts for shp::ShpFacts {
    fn has_identity(&self) -> bool {
        shp::ShpFacts::any(self)
    }
}

impl HasFacts for scr::ScrFacts {
    fn has_identity(&self) -> bool {
        scr::ScrFacts::any(self)
    }
}

impl HasFacts for rpt::RptFacts {
    fn has_identity(&self) -> bool {
        rpt::RptFacts::any(self)
    }
}

impl HasFacts for ic::IcFacts {
    fn has_identity(&self) -> bool {
        ic::IcFacts::any(self)
    }
}

impl HasFacts for it::ItFacts {
    fn has_identity(&self) -> bool {
        it::ItFacts::any(self)
    }
}

impl HasFacts for iff::IffFacts {
    fn has_identity(&self) -> bool {
        iff::IffFacts::any(self)
    }
}

impl HasFacts for fr::FrFacts {
    fn has_identity(&self) -> bool {
        fr::FrFacts::any(self)
    }
}

async fn get_xml(client: &reqwest::Client, url: &str) -> anyhow::Result<String> {
    let response = client
        .get(url)
        .header(
            "Accept",
            "application/xml, application/xhtml+xml, text/xml, */*",
        )
        .header("Referer", NSE_RSS_PAGE)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await?;
    let status = response.status();
    let bytes = response.bytes().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}");
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

async fn record_status(
    state: &NseState,
    kind: NseFeedKind,
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
        warn!(error = %e, feed = kind.slug(), "failed to write NSE feed status");
    }
}
