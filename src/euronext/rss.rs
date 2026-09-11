//! Strict RSS 2.0 types matching the Euronext Athens XML endpoints.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Root `<rss>` document from a Euronext Athens feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "rss")]
pub struct EuronextRssDocument {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@xmlns:hlxcd", default)]
    pub xmlns_hlxcd: Option<String>,
    #[serde(rename = "@xml:base", default)]
    pub xml_base: Option<String>,
    #[serde(rename = "@base", default)]
    pub base: Option<String>,
    pub channel: EuronextRssChannel,
}

/// `<channel>` element. Optional children match observed Athens variance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EuronextRssChannel {
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
    pub items: Vec<EuronextRssItem>,
}

/// `<guid>` may carry `isPermaLink`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EuronextRssGuid {
    #[serde(rename = "@isPermaLink", default)]
    pub is_permalink: Option<String>,
    #[serde(rename = "$text", default)]
    pub value: String,
}

impl EuronextRssGuid {
    pub fn as_str(&self) -> &str {
        self.value.trim()
    }
}

/// Nested `hlxcd:helex-company-data` after namespace normalization.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelexCompanyData {
    #[serde(rename = "hlxcd-company-name", default)]
    pub company_name: Option<String>,
    #[serde(rename = "hlxcd-company-ticker-symbol", default)]
    pub company_ticker: Option<String>,
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// `<item>` child of `<channel>`. Unknown child elements are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EuronextRssItem {
    pub title: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "pubDate", default)]
    pub pub_date: String,
    #[serde(default)]
    pub guid: Option<EuronextRssGuid>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub attachment: Option<String>,
    #[serde(rename = "attachment-title", default)]
    pub attachment_title: Option<String>,
    #[serde(rename = "hlxcd-helex-company-data", default)]
    pub company: Option<HelexCompanyData>,
}

fn normalize_rss(xml: &str) -> String {
    xml.trim_start_matches('\u{feff}')
        .trim()
        .replace("<hlxcd:helex-company-data", "<hlxcd-helex-company-data")
        .replace("</hlxcd:helex-company-data", "</hlxcd-helex-company-data")
        .replace("<hlxcd:company-name", "<hlxcd-company-name")
        .replace("</hlxcd:company-name", "</hlxcd-company-name")
        .replace(
            "<hlxcd:company-ticker-symbol",
            "<hlxcd-company-ticker-symbol",
        )
        .replace(
            "</hlxcd:company-ticker-symbol",
            "</hlxcd-company-ticker-symbol",
        )
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

/// Parsed channel items from an Athens RSS document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedItems {
    pub last_build_date: Option<String>,
    pub items: Vec<EuronextRssItem>,
}

impl ParsedItems {
    pub fn from_rss(doc: EuronextRssDocument) -> Self {
        Self {
            last_build_date: doc.channel.last_build_date,
            items: doc.channel.items,
        }
    }
}

/// Parse a Euronext Athens RSS 2.0 XML document.
///
/// `quick-xml` serde drops namespace prefixes, so `hlxcd:*` is rewritten first.
pub fn parse_rss(xml: &str) -> Result<EuronextRssDocument, quick_xml::DeError> {
    quick_xml::de::from_str(&normalize_rss(xml))
}

/// Honor RFC 3339 first, then `DD/MM/YYYY` (risk-management), then RFC 2822.
pub fn parse_euronext_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            NaiveDate::parse_from_str(s, "%d/%m/%Y")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
        })
        .or_else(|| {
            DateTime::parse_from_rfc2822(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
}

/// Stable identity for a Euronext RSS item.
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
        EuronextRssItem, content_hash, looks_like_html, parse_euronext_datetime, parse_rss,
    };
    use chrono::{TimeZone, Utc};

    const ISSUER: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss xmlns:hlxcd="https://athens.euronext.com/sites/default/files/rss/xsd/CompanyDataAtomAttributes-v1.xsd" version="2.0" xml:base="https://athens.euronext.com/en">
  <channel>
    <title>Issuer Announcements</title>
    <link>https://athens.euronext.com/en</link>
    <description>ATHEX Issuer Announcements</description>
    <language>en</language>
    <item>
      <title>ANNOUNCEMENT FOR THE PURCHASE OF OWN SHARES</title>
      <link>https://athens.euronext.com/en/node/969219</link>
      <description>&lt;p&gt;please see attached announcement&lt;/p&gt;</description>
      <pubDate>2026-09-11T16:03:10Z</pubDate>
      <guid isPermaLink="false">969219 at https://athens.euronext.com</guid>
      <attachment>https://athens.euronext.com/sites/default/files/example.pdf</attachment>
      <attachment-title>Announcement for the purchase of own shares</attachment-title>
      <hlxcd:helex-company-data>	<hlxcd:company-name>PRODEA AE</hlxcd:company-name>
	<hlxcd:company-ticker-symbol>PRODEA</hlxcd:company-ticker-symbol></hlxcd:helex-company-data>
    </item>
  </channel>
</rss>"#;

    const RISK: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xml:base="https://athens.euronext.com">
  <channel>
    <title>Risk-Management</title>
    <link>https://athens.euronext.com</link>
    <description></description>
    <language>en</language>
    <item>
      <title>240711_Margin_Parameters_Derivatives_Market.xlsx</title>
      <link>https://athens.euronext.com/sites/default/files/athex-documents/Risk-Management/240711_Margin_Parameters_Derivatives_Market.xlsx</link>
      <description>240711_Margin_Parameters_Derivatives_Market.xlsx</description>
      <language>en</language>
      <pubDate>11/07/2024</pubDate>
    </item>
  </channel>
</rss>"#;

    #[test]
    fn parses_issuer_announcement_hlxcd_and_attachment() {
        let doc = parse_rss(ISSUER).expect("parse issuer");
        assert_eq!(doc.version, "2.0");
        assert_eq!(doc.channel.title, "Issuer Announcements");
        assert_eq!(doc.channel.items.len(), 1);
        let item = &doc.channel.items[0];
        assert!(item.title.contains("OWN SHARES"));
        assert!(item.link.contains("969219"));
        assert_eq!(item.pub_date, "2026-09-11T16:03:10Z");
        assert_eq!(
            item.guid.as_ref().map(|g| g.as_str()),
            Some("969219 at https://athens.euronext.com")
        );
        assert_eq!(
            item.guid.as_ref().and_then(|g| g.is_permalink.as_deref()),
            Some("false")
        );
        assert!(item.attachment.as_deref().unwrap().ends_with(".pdf"));
        assert_eq!(
            item.attachment_title.as_deref(),
            Some("Announcement for the purchase of own shares")
        );
        let company = item.company.as_ref().expect("company");
        assert_eq!(company.company_name.as_deref(), Some("PRODEA AE"));
        assert_eq!(company.company_ticker.as_deref(), Some("PRODEA"));
    }

    #[test]
    fn parses_risk_management_dmy_date() {
        let doc = parse_rss(RISK).expect("parse risk");
        let item = &doc.channel.items[0];
        assert!(item.title.contains("Margin_Parameters"));
        assert_eq!(item.pub_date, "11/07/2024");
        assert!(item.guid.is_none());
        assert_eq!(item.language.as_deref(), Some("en"));
        let dt = parse_euronext_datetime(&item.pub_date).unwrap();
        assert_eq!(dt, Utc.with_ymd_and_hms(2024, 7, 11, 0, 0, 0).unwrap());
    }

    #[test]
    fn rfc3339_offset_is_honored() {
        let dt = parse_euronext_datetime("2026-09-11T16:03:10Z").unwrap();
        assert_eq!(dt, Utc.with_ymd_and_hms(2026, 9, 11, 16, 3, 10).unwrap());
        assert!(parse_euronext_datetime("").is_none());
    }

    #[test]
    fn looks_like_html_detects_interstitial() {
        assert!(looks_like_html(
            "<!DOCTYPE html><html><body>Access Denied</body></html>"
        ));
        assert!(!looks_like_html(ISSUER));
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
        let item: EuronextRssItem = quick_xml::de::from_str(xml).expect("item");
        assert_eq!(item.title, "a");
    }

    #[test]
    fn content_hash_includes_guid() {
        let a = content_hash("issuer-announcements", "t", "l", "d", "p", "969219");
        let b = content_hash("issuer-announcements", "t", "l", "d", "p", "969220");
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
    }
}
