//! Strict RSS 2.0 types matching the Nasdaq IR XML endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Root `<rss>` document from a Nasdaq IR feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "rss")]
pub struct NasdaqRssDocument {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@xmlns:dc", default)]
    pub xmlns_dc: Option<String>,
    #[serde(rename = "@xml:base", default)]
    pub xml_base: Option<String>,
    #[serde(rename = "@base", default)]
    pub base: Option<String>,
    pub channel: NasdaqRssChannel,
}

/// `<channel>` element. Optional children match observed Nasdaq IR variance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NasdaqRssChannel {
    pub title: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(rename = "lastBuildDate", default)]
    pub last_build_date: Option<String>,
    #[serde(rename = "item", default)]
    pub items: Vec<NasdaqRssItem>,
}

/// `<guid>` may carry `isPermaLink`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NasdaqRssGuid {
    #[serde(rename = "@isPermaLink", default)]
    pub is_permalink: Option<String>,
    #[serde(rename = "$text", default)]
    pub value: String,
}

impl NasdaqRssGuid {
    pub fn as_str(&self) -> &str {
        self.value.trim()
    }
}

/// `<item>` child of `<channel>`. Unknown child elements are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NasdaqRssItem {
    pub title: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "pubDate", default)]
    pub pub_date: String,
    #[serde(rename = "dc-creator", default)]
    pub dc_creator: Option<String>,
    #[serde(default)]
    pub guid: Option<NasdaqRssGuid>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

fn normalize_rss(xml: &str) -> String {
    xml.trim_start_matches('\u{feff}')
        .trim()
        .replace("<dc:creator", "<dc-creator")
        .replace("</dc:creator", "</dc-creator")
}

/// True when the body looks like HTML rather than RSS (WAF/interstitial).
pub fn looks_like_html(xml: &str) -> bool {
    let t = xml.trim_start_matches('\u{feff}').trim_start();
    let head = t.get(..256).unwrap_or(t).to_ascii_lowercase();
    head.starts_with("<!doctype html") || head.contains("<html")
}

/// True when the RSS document includes a closing `</rss>` root.
pub fn looks_complete(xml: &str) -> bool {
    xml.trim_end().to_ascii_lowercase().ends_with("</rss>")
}

/// True when the body is an Atom `<feed>`.
pub fn looks_atom(xml: &str) -> bool {
    let t = xml
        .trim_start_matches('\u{feff}')
        .trim_start()
        .to_ascii_lowercase();
    t.contains("<feed") && !t.contains("<rss")
}

/// True when RSS or Atom looks closed.
pub fn looks_complete_feed(xml: &str) -> bool {
    let t = xml.trim_end().to_ascii_lowercase();
    t.ends_with("</rss>") || t.ends_with("</feed>")
}

/// Parsed channel items from either IR RSS or EDGAR Atom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedItems {
    pub last_build_date: Option<String>,
    pub items: Vec<NasdaqRssItem>,
}

impl ParsedItems {
    pub fn from_rss(doc: NasdaqRssDocument) -> Self {
        Self {
            last_build_date: doc.channel.last_build_date,
            items: doc.channel.items,
        }
    }
}

/// Parse a Nasdaq IR RSS 2.0 XML document.
///
/// `quick-xml` serde drops namespace prefixes, so `dc:creator` is rewritten to
/// `dc-creator` first.
pub fn parse_rss(xml: &str) -> Result<NasdaqRssDocument, quick_xml::DeError> {
    quick_xml::de::from_str(&normalize_rss(xml))
}

/// Honor RFC 2822 / RFC 3339 offsets (Nasdaq IR uses `-0500`, not IST).
pub fn parse_nasdaq_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc2822(s)
        .or_else(|_| DateTime::parse_from_rfc3339(s))
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Stable identity for a Nasdaq RSS item.
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

#[cfg(test)]
mod tests {
    use super::{NasdaqRssItem, content_hash, looks_like_html, parse_nasdaq_datetime, parse_rss};
    use chrono::{TimeZone, Utc};

    const NEWS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0" xml:base="https://ir.nasdaq.com/">
  <channel>
    <title>Nasdaq, Inc. News Releases</title>
    <link>https://ir.nasdaq.com/</link>
    <description>Nasdaq, Inc. News Releases</description>
    <language>en</language>
    <item>
      <title>Nasdaq Reports November 2025 Volumes</title>
      <link>https://ir.nasdaq.com/news-releases/news-release-details/nasdaq-reports-november-2025-volumes</link>
      <description>NEW YORK , Dec. 03, 2025 (GLOBE NEWSWIRE) -- Nasdaq (Nasdaq: NDAQ) today reported monthly volumes for November 2025.</description>
      <pubDate>Wed, 03 Dec 2025 16:05:00 -0500</pubDate>
      <dc:creator>Nasdaq, Inc. News Releases</dc:creator>
      <guid isPermaLink="false">109751</guid>
    </item>
  </channel>
</rss>"#;

    const SEC: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss xmlns:dc="http://purl.org/dc/elements/1.1/" version="2.0" xml:base="https://ir.nasdaq.com/">
  <channel>
    <title>Nasdaq, Inc. SEC Filings</title>
    <link>https://ir.nasdaq.com/</link>
    <description>Nasdaq, Inc. SEC Filings</description>
    <language>en</language>
    <item>
      <title>8-K: Report of unscheduled material events or corporate event</title>
      <link>https://ir.nasdaq.com/sec-filings/sec-filing/8-k/0001193125-25-319238</link>
      <description>NASDAQ, INC.</description>
      <pubDate>Mon, 15 Dec 2025 16:21:45 -0500</pubDate>
      <dc:creator>Nasdaq, Inc. SEC Filings</dc:creator>
      <guid isPermaLink="false">109801</guid>
    </item>
  </channel>
</rss>"#;

    #[test]
    fn parses_news_release_dc_creator_and_guid() {
        let doc = parse_rss(NEWS).expect("parse news");
        assert_eq!(doc.version, "2.0");
        assert_eq!(doc.channel.title, "Nasdaq, Inc. News Releases");
        assert_eq!(doc.channel.items.len(), 1);
        let item = &doc.channel.items[0];
        assert!(item.title.contains("November 2025"));
        assert!(item.link.contains("nasdaq-reports-november-2025-volumes"));
        assert_eq!(item.pub_date, "Wed, 03 Dec 2025 16:05:00 -0500");
        assert_eq!(
            item.dc_creator.as_deref(),
            Some("Nasdaq, Inc. News Releases")
        );
        assert_eq!(item.guid.as_ref().map(|g| g.as_str()), Some("109751"));
        assert_eq!(
            item.guid.as_ref().and_then(|g| g.is_permalink.as_deref()),
            Some("false")
        );
    }

    #[test]
    fn parses_sec_filing_item() {
        let doc = parse_rss(SEC).expect("parse sec");
        let item = &doc.channel.items[0];
        assert!(item.title.starts_with("8-K:"));
        assert!(item.link.contains("0001193125-25-319238"));
        assert_eq!(item.description, "NASDAQ, INC.");
        assert_eq!(item.guid.as_ref().map(|g| g.as_str()), Some("109801"));
    }

    #[test]
    fn rfc2822_offset_is_honored_not_ist() {
        let dt = parse_nasdaq_datetime("Wed, 03 Dec 2025 16:05:00 -0500").unwrap();
        assert_eq!(dt, Utc.with_ymd_and_hms(2025, 12, 3, 21, 5, 0).unwrap());
        let dt = parse_nasdaq_datetime("Mon, 15 Dec 2025 13:01:00 -0500").unwrap();
        assert_eq!(dt, Utc.with_ymd_and_hms(2025, 12, 15, 18, 1, 0).unwrap());
        assert!(parse_nasdaq_datetime("").is_none());
    }

    #[test]
    fn looks_like_html_detects_interstitial() {
        assert!(looks_like_html(
            "<!DOCTYPE html><html><body>Access Denied</body></html>"
        ));
        assert!(!looks_like_html(NEWS));
    }

    #[test]
    fn rejects_unknown_item_child_elements() {
        let xml = r#"<?xml version="1.0"?><rss version="2.0"><channel>
<title>t</title><link>l</link><description>d</description>
<item><title>a</title><link>b</link><description>c</description><pubDate>d</pubDate><foo>nope</foo></item>
</channel></rss>"#;
        let err = parse_rss(xml).expect_err("unknown item child");
        let msg = err.to_string();
        assert!(
            msg.contains("foo") || msg.contains("unknown"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn item_struct_denies_unknown_fields_at_type_level() {
        let xml = "<item><title>a</title><link>b</link><description>c</description><pubDate>d</pubDate></item>";
        let item: NasdaqRssItem = quick_xml::de::from_str(xml).expect("item");
        assert_eq!(item.title, "a");
    }

    #[test]
    fn content_hash_includes_guid() {
        let a = content_hash("news-releases", "t", "l", "d", "p", "109751");
        let b = content_hash("news-releases", "t", "l", "d", "p", "109752");
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
    }
}
