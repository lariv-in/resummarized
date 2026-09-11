//! Strict RSS 2.0 types matching the JPX English XML endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Root `<rss>` document from a JPX English feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "rss")]
pub struct JpxRssDocument {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@xmlns:dc", default)]
    pub xmlns_dc: Option<String>,
    #[serde(rename = "@xmlns:sy", default)]
    pub xmlns_sy: Option<String>,
    #[serde(rename = "@xmlns:admin", default)]
    pub xmlns_admin: Option<String>,
    #[serde(rename = "@xmlns:rdf", default)]
    pub xmlns_rdf: Option<String>,
    pub channel: JpxRssChannel,
}

/// `<channel>` element. Optional children match observed JPX variance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JpxRssChannel {
    pub title: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "pubDate", default)]
    pub pub_date: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(rename = "item", default)]
    pub items: Vec<JpxRssItem>,
}

/// `<guid>` may carry `isPermaLink`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JpxRssGuid {
    #[serde(rename = "@isPermaLink", default)]
    pub is_permalink: Option<String>,
    #[serde(rename = "$text", default)]
    pub value: String,
}

impl JpxRssGuid {
    pub fn as_str(&self) -> &str {
        self.value.trim()
    }
}

/// `<item>` child of `<channel>`. Unknown child elements are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JpxRssItem {
    pub title: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "pubDate", default)]
    pub pub_date: String,
    #[serde(default)]
    pub guid: Option<JpxRssGuid>,
}

fn normalize_rss(xml: &str) -> String {
    xml.trim_start_matches('\u{feff}').trim().to_string()
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

/// Parsed channel items from a JPX RSS document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedItems {
    pub last_build_date: Option<String>,
    pub items: Vec<JpxRssItem>,
}

impl ParsedItems {
    pub fn from_rss(doc: JpxRssDocument) -> Self {
        Self {
            last_build_date: doc.channel.pub_date.filter(|s| !s.trim().is_empty()),
            items: doc.channel.items,
        }
    }
}

/// Parse a JPX English RSS 2.0 XML document.
pub fn parse_rss(xml: &str) -> Result<JpxRssDocument, quick_xml::DeError> {
    quick_xml::de::from_str(&normalize_rss(xml))
}

/// Honor RFC 2822 offsets first (JPX uses `+0900`), then RFC 3339.
pub fn parse_jpx_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc2822(s)
        .or_else(|_| DateTime::parse_from_rfc3339(s))
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Stable identity for a JPX RSS item. Includes `guid` because halt items often
/// share the same listing-page URL.
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
    use super::{
        JpxRssItem, ParsedItems, content_hash, looks_like_html, parse_jpx_datetime, parse_rss,
    };
    use chrono::{TimeZone, Utc};

    const MARKET_NEWS: &str = r#"<rss xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:sy="http://purl.org/rss/1.0/modules/syndication/" xmlns:admin="http://webns.net/mvcb/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" version="2.0">
  <channel>
    <title>JPX Market News</title>
    <link>https://www.jpx.co.jp/english/rss/markets_news.xml</link>
    <description/>
    <pubDate>Fri, 11 Sep 2026 16:00:00 +0900</pubDate>
    <category>markets_news</category>
    <language>en-us</language>
      <item>
        <title>
[TSE]Approval of Initial Listing (TOKYO PRO Market): HIGHNESSCORPORATION CO.,LTD.</title>
        <link>https://www.jpx.co.jp/english/equities/products/tpm/issues/index.html</link>
        <guid isPermaLink="false">markets_newsvk0khi0000028is31789110000000</guid>
        <pubDate>Fri, 11 Sep 2026 16:00:00 +0900</pubDate>
      </item>
  </channel>
</rss>"#;

    const EMPTY_NEWS: &str = r#"<rss xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:sy="http://purl.org/rss/1.0/modules/syndication/" xmlns:admin="http://webns.net/mvcb/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" version="2.0">
  <channel>
    <title>JPX News Release</title>
    <link>https://www.jpx.co.jp/english/rss/jpx-news.xml</link>
    <description/>
    <pubDate></pubDate>
    <category>jpx-news</category>
    <language>en-us</language>
  </channel>
</rss>
"#;

    #[test]
    fn parses_market_news_guid_and_jst_date() {
        let doc = parse_rss(MARKET_NEWS).expect("parse market news");
        assert_eq!(doc.version, "2.0");
        assert_eq!(doc.channel.title, "JPX Market News");
        assert_eq!(doc.channel.category.as_deref(), Some("markets_news"));
        assert_eq!(doc.channel.language.as_deref(), Some("en-us"));
        assert_eq!(doc.channel.items.len(), 1);
        let item = &doc.channel.items[0];
        assert!(item.title.contains("HIGHNESSCORPORATION"));
        assert!(item.title.starts_with('\n') || item.title.starts_with('['));
        assert!(item.link.contains("tpm/issues"));
        assert_eq!(item.description, "");
        assert_eq!(item.pub_date, "Fri, 11 Sep 2026 16:00:00 +0900");
        assert_eq!(
            item.guid.as_ref().map(|g| g.as_str()),
            Some("markets_newsvk0khi0000028is31789110000000")
        );
        assert_eq!(
            item.guid.as_ref().and_then(|g| g.is_permalink.as_deref()),
            Some("false")
        );
        let parsed = ParsedItems::from_rss(doc);
        assert_eq!(
            parsed.last_build_date.as_deref(),
            Some("Fri, 11 Sep 2026 16:00:00 +0900")
        );
    }

    #[test]
    fn parses_empty_news_release_channel() {
        let doc = parse_rss(EMPTY_NEWS).expect("parse empty news");
        assert_eq!(doc.channel.title, "JPX News Release");
        assert!(doc.channel.items.is_empty());
        let parsed = ParsedItems::from_rss(doc);
        assert!(parsed.last_build_date.is_none());
        assert!(parsed.items.is_empty());
    }

    #[test]
    fn rfc2822_plus0900_is_honored() {
        let dt = parse_jpx_datetime("Fri, 11 Sep 2026 16:00:00 +0900").unwrap();
        assert_eq!(dt, Utc.with_ymd_and_hms(2026, 9, 11, 7, 0, 0).unwrap());
        assert!(parse_jpx_datetime("").is_none());
        assert!(parse_jpx_datetime("   ").is_none());
    }

    #[test]
    fn looks_like_html_detects_interstitial() {
        assert!(looks_like_html(
            "<!DOCTYPE html><html><body>Access Denied</body></html>"
        ));
        assert!(!looks_like_html(MARKET_NEWS));
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
        let item: JpxRssItem = quick_xml::de::from_str(xml).expect("item");
        assert_eq!(item.title, "a");
    }

    #[test]
    fn content_hash_includes_guid() {
        let a = content_hash("equities-halt", "t", "l", "d", "p", "guid-1");
        let b = content_hash("equities-halt", "t", "l", "d", "p", "guid-2");
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
    }
}
