//! Strict RSS 2.0 types matching the NSE XML endpoints.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Root `<rss>` document from an NSE feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "rss")]
pub struct NseRssDocument {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@xmlns:atom", default)]
    pub xmlns_atom: Option<String>,
    pub channel: NseRssChannel,
}

/// `<channel>` element. Optional children match observed NSE variance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NseRssChannel {
    #[serde(rename = "atom-link", default)]
    pub atom_link: Option<NseRssAtomLink>,
    pub title: String,
    pub link: String,
    pub description: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(rename = "lastBuildDate", default)]
    pub last_build_date: Option<String>,
    #[serde(default)]
    pub ttl: Option<String>,
    #[serde(default)]
    pub image: Option<NseRssImage>,
    #[serde(rename = "item", default)]
    pub items: Vec<NseRssItem>,
}

/// `<atom:link>` on the channel (`rel="self"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NseRssAtomLink {
    #[serde(rename = "@href")]
    pub href: String,
    #[serde(rename = "@rel", default)]
    pub rel: Option<String>,
    #[serde(rename = "@type", default)]
    pub r#type: Option<String>,
}

/// `<image>` child of `<channel>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NseRssImage {
    pub title: String,
    pub link: String,
    pub url: String,
    #[serde(default)]
    pub width: Option<String>,
    #[serde(default)]
    pub height: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// `<item>` child of `<channel>`. Unknown child elements are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NseRssItem {
    pub title: String,
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "pubDate", default)]
    pub pub_date: String,
}

fn normalize_rss(xml: &str) -> String {
    xml.trim_start_matches('\u{feff}')
        .trim()
        .replace("<atom:link", "<atom-link")
        .replace("</atom:link", "</atom-link")
}

/// True when the body looks like HTML rather than RSS (WAF/interstitial).
pub fn looks_like_html(xml: &str) -> bool {
    let t = xml.trim_start_matches('\u{feff}').trim_start();
    let head = t.get(..256).unwrap_or(t).to_ascii_lowercase();
    head.starts_with("<!doctype html") || head.contains("<html")
}

/// True when the RSS document includes a closing `</rss>` root.
pub fn looks_complete(xml: &str) -> bool {
    xml.trim_end().ends_with("</rss>")
}

fn parse_rss_strict(normalized: &str) -> Result<NseRssDocument, quick_xml::DeError> {
    quick_xml::de::from_str(normalized)
}

/// Recover complete `<item>` elements from a truncated or otherwise ill-formed feed.
fn salvage_rss(normalized: &str) -> Option<NseRssDocument> {
    let mut items = Vec::new();
    let mut from = 0;
    while let Some(rel) = normalized[from..].find("<item>") {
        let start = from + rel;
        let Some(rel_end) = normalized[start..].find("</item>") else {
            break;
        };
        let end = start + rel_end + "</item>".len();
        if let Ok(item) = quick_xml::de::from_str::<NseRssItem>(&normalized[start..end]) {
            items.push(item);
        }
        from = end;
    }
    if items.is_empty() {
        return None;
    }
    let header = normalized.find("<item>").map(|i| &normalized[..i])?;
    let wrapped = format!("{header}</channel></rss>");
    let mut doc = parse_rss_strict(&wrapped).ok()?;
    doc.channel.items = items;
    Some(doc)
}

/// Parse an NSE RSS 2.0 XML document.
///
/// `quick-xml` serde drops namespace prefixes, so `atom:link` would collide with
/// channel `<link>`. Rewrite the namespaced element to `atom-link` first.
///
/// NSE rewrites `Online_announcements.xml` in place; a fetch can land on a
/// truncated copy (`</link>` never arrives). HTML interstitials also contain
/// void `<link>` tags that produce the same error. Complete `<item>`s are
/// salvaged when the strict parse fails.
pub fn parse_rss(xml: &str) -> Result<NseRssDocument, quick_xml::DeError> {
    if looks_like_html(xml) {
        return Err(quick_xml::DeError::Custom("html document".into()));
    }
    let normalized = normalize_rss(xml);
    match parse_rss_strict(&normalized) {
        Ok(doc) => Ok(doc),
        Err(e) => match salvage_rss(&normalized) {
            Some(doc) => {
                tracing::warn!(
                    items = doc.channel.items.len(),
                    error = %e,
                    "salvaged complete NSE RSS items from ill-formed document"
                );
                Ok(doc)
            }
            None => Err(e),
        },
    }
}

/// Stable identity for an NSE RSS item (no `guid` in the schema).
pub fn content_hash(
    feed_kind: &str,
    title: &str,
    link: &str,
    description: &str,
    pub_date: &str,
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
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{NseRssItem, parse_rss};

    const ANNOUNCEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">
<channel>
<atom:link href="http://www.nseindia.com/content/RSS/Online_announcements.xml" rel="self" type="application/rss+xml"/>
<link>https://www.nseindia.com/companies-listing/corporate-filings-announcements</link>
<title>NSE News - Latest Announcements</title>
<description>National Stock Exchange- Announcements</description>
<language>en-us</language>
<lastBuildDate>Mon, 07 Sep 2026 23:29:39 +0530</lastBuildDate>
<ttl>5</ttl>
<image>
<title>Latest Announcements</title>
<link>https://www.nseindia.com/companies-listing/corporate-filings-announcements</link>
<url>https://www.nseindia.com/assets/images/NSE_Logo.svg</url>
<width>122</width>
<height>42</height>
<description>National Stock Exchange of India</description>
</image>
<item>
<title>Ajax Engineering Limited</title>
<link>https://nsearchives.nseindia.com/corporate/AJAXENGG_ROID_98099_KMP_Doc.zip</link>
<description>Ajax Engineering Limited has informed the Exchange about Resignation of Director/KMP/SMP |SUBJECT: Resignation of Director/KMP/SMP</description>
<pubDate>07-Sep-2026 23:28:05</pubDate>
</item>
</channel>
</rss>
"#;

    const CIRCULARS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">
<channel>
<atom:link href="http://www.nseindia.com/content/RSS/Circulars.xml" rel="self" type="application/rss+xml"/>
<title>NSE Circulars </title>
<link>https://www.nseindia.com/resources/exchange-communication-circulars</link>
<description>National Stock Exchange - Circulars Issued by Exchange</description>
<language>en-us</language>
<lastBuildDate>Mon, 7 Sep 2026 23:30:32 +0530</lastBuildDate>
<ttl>5</ttl>
<image>
<title>NSE Circulars</title>
<link>https://www.nseindia.com/resources/exchange-communication-circulars</link>
<url>https://www.nseindia.com/assets/images/NSE_Logo.svg</url>
<width>122</width>
<height>42</height>
<description>National Stock Exchange of India</description>
</image>
<item>
<title>Listing of Equity Shares of Rays of Belief Limited (IPO)
</title>
<link>https://nsearchives.nseindia.com/content/circulars/CML76229.zip</link>
<description/>
<pubDate>Mon, 7 Sep 2026 00:00:00 +0530</pubDate>
</item>
</channel>
</rss>
"#;

    const CORPORATE_ACTIONS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
<title>NSE News - Latest Corporates Action</title>
<link>https://www.nseindia.com/companies-listing/corporate-filings-actions</link>
<description>National Stock Exchange- Corporate Actions</description>
<language>en-us</language>
<lastBuildDate>Mon, 07 Sep 2026 06:09:03 +0530</lastBuildDate>
<ttl>5</ttl>
<image>
<title>Corporate Actions</title>
<link>https://www.nseindia.com/companies-listing/corporate-filings-actions</link>
<url>https://www.nseindia.com/assets/images/NSE_Logo.svg</url>
<width>122</width>
<height>42</height>
<description>National Stock Exchange of India</description>
</image>
<item>
<title>Caplin Point Laboratories Limited - Ex-Date: 18-Sep-2026
</title>
<link>https://www.nseindia.com/companies-listing/corporate-filings-actions</link>
<description>SERIES:EQ |PURPOSE:DIVIDEND - RS 4 PER SHARE |FACE VALUE:2 |RECORD DATE:18-Sep-2026 |BOOK CLOSURE START DATE:- |BOOK CLOSURE END DATE:-</description>
<pubDate>07-Sep-2026 06:09:03</pubDate>
</item>
</channel>
</rss>
"#;

    #[test]
    fn parses_announcements_item_fields() {
        let doc = parse_rss(ANNOUNCEMENTS).expect("parse announcements");
        assert_eq!(doc.version, "2.0");
        assert_eq!(
            doc.xmlns_atom.as_deref(),
            Some("http://www.w3.org/2005/Atom")
        );
        let atom = doc.channel.atom_link.expect("atom:link");
        assert_eq!(atom.rel.as_deref(), Some("self"));
        assert_eq!(doc.channel.items.len(), 1);
        let item = &doc.channel.items[0];
        assert_eq!(item.title, "Ajax Engineering Limited");
        assert_eq!(item.pub_date, "07-Sep-2026 23:28:05");
        assert!(item.description.contains("|SUBJECT:"));
        assert_eq!(doc.channel.ttl.as_deref(), Some("5"));
        let image = doc.channel.image.expect("image");
        assert_eq!(image.width.as_deref(), Some("122"));
    }

    #[test]
    fn parses_circulars_empty_description_and_rfc822_pubdate() {
        let doc = parse_rss(CIRCULARS).expect("parse circulars");
        let item = &doc.channel.items[0];
        assert_eq!(item.description, "");
        assert_eq!(item.pub_date, "Mon, 7 Sep 2026 00:00:00 +0530");
        assert!(item.title.contains("Rays of Belief"));
    }

    #[test]
    fn parses_corporate_actions_without_atom_link() {
        let doc = parse_rss(CORPORATE_ACTIONS).expect("parse corporate actions");
        assert!(doc.channel.atom_link.is_none());
        assert_eq!(doc.channel.items.len(), 1);
        assert!(
            doc.channel.items[0]
                .description
                .starts_with("SERIES:EQ |PURPOSE:")
        );
    }

    #[test]
    fn rejects_unknown_item_child_elements() {
        let xml = r#"<?xml version="1.0"?><rss version="2.0"><channel>
<title>t</title><link>l</link><description>d</description>
<item><title>a</title><link>b</link><description>c</description><pubDate>d</pubDate><guid>nope</guid></item>
</channel></rss>"#;
        let err = parse_rss(xml).expect_err("guid is not in the NSE item schema");
        let msg = err.to_string();
        assert!(
            msg.contains("guid") || msg.contains("unknown"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn item_struct_denies_unknown_fields_at_type_level() {
        let xml = "<item><title>a</title><link>b</link><description>c</description><pubDate>d</pubDate></item>";
        let item: NseRssItem = quick_xml::de::from_str(xml).expect("item");
        assert_eq!(item.title, "a");
    }

    /// Live announcements feed uses empty `<link/>` on NAV/ETF items and a stray
    /// `<text/>` after `</channel>`.
    const ANNOUNCEMENTS_EMPTY_LINK: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">
<channel>
<atom:link href="http://www.nseindia.com/content/RSS/Online_announcements.xml" rel="self" type="application/rss+xml"/>
<link>https://www.nseindia.com/companies-listing/corporate-filings-announcements</link>
<title>NSE News - Latest Announcements</title>
<description>National Stock Exchange- Announcements</description>
<language>en-us</language>
<lastBuildDate>Tue, 08 Sep 2026 14:35:11 +0530</lastBuildDate>
<ttl>5</ttl>
<image>
<title>Latest Announcements</title>
<link>https://www.nseindia.com/companies-listing/corporate-filings-announcements</link>
<url>https://www.nseindia.com/assets/images/NSE_Logo.svg</url>
<width>122</width>
<height>42</height>
<description>National Stock Exchange of India</description>
</image>
<item>
<title>Zerodha Fund House - Zerodha Nifty Midcap 150 ETF</title>
<link/>
<description>Zerodha Asset Management Private Limited has informed the Exchange that the Net Asset Value (per unit) of Zerodha Fund House - Zerodha Nifty Midcap 150 ETF as on September 07, 2026 is Rs. 11.6173. |SUBJECT: Declaration of NAV</description>
<pubDate>08-Sep-2026 13:05:00</pubDate>
</item>
</channel>
<text/>
</rss>
"#;

    #[test]
    fn parses_empty_self_closing_item_link_and_trailing_text() {
        let doc = parse_rss(ANNOUNCEMENTS_EMPTY_LINK).expect("parse announcements with empty link");
        assert_eq!(doc.channel.items.len(), 1);
        let item = &doc.channel.items[0];
        assert_eq!(item.link, "");
        assert!(item.description.contains("SUBJECT:"));
    }

    #[test]
    fn salvages_complete_items_from_truncated_feed() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">
<channel>
<atom:link href="http://www.nseindia.com/content/RSS/Online_announcements.xml" rel="self" type="application/rss+xml"/>
<link>https://www.nseindia.com/companies-listing/corporate-filings-announcements</link>
<title>NSE News - Latest Announcements</title>
<description>National Stock Exchange- Announcements</description>
<item>
<title>Ajax Engineering Limited</title>
<link>https://nsearchives.nseindia.com/corporate/AJAXENGG_ROID_98099_KMP_Doc.zip</link>
<description>Ajax Engineering Limited has informed the Exchange |SUBJECT: Resignation</description>
<pubDate>07-Sep-2026 23:28:05</pubDate>
</item>
<item>
<title>Truncated Company</title>
<link>https://nsearchives.nseindia.com/corporate/PARTIAL"#;
        assert!(!super::looks_complete(xml));
        let err = super::parse_rss_strict(&super::normalize_rss(xml)).expect_err("truncated");
        let msg = err.to_string();
        assert!(
            msg.contains("</link>") || msg.contains("ill-formed"),
            "unexpected error: {msg}"
        );
        let doc = parse_rss(xml).expect("salvage truncated announcements");
        assert_eq!(doc.channel.items.len(), 1);
        assert_eq!(doc.channel.items[0].title, "Ajax Engineering Limited");
    }

    #[test]
    fn rejects_html_interstitial() {
        let html = r#"<!DOCTYPE html><html><head><link rel="stylesheet" href="/x.css"><title>NSE</title></head><body>blocked</body></html>"#;
        let err = parse_rss(html).expect_err("html");
        assert!(err.to_string().contains("html"), "{err}");
    }
}
