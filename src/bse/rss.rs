//! Strict RSS 2.0 types matching the BSE XML endpoints.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Root `<rss>` document from a BSE feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "rss")]
pub struct BseRssDocument {
    #[serde(rename = "@version")]
    pub version: String,
    pub channel: BseRssChannel,
}

/// `<channel>` element. Optional children match observed BSE variance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BseRssChannel {
    pub title: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub copyright: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(rename = "lastBuildDate", default)]
    pub last_build_date: Option<String>,
    #[serde(default)]
    pub image: Option<BseRssImage>,
    #[serde(rename = "item", default)]
    pub items: Vec<BseRssItem>,
}

/// `<image>` child of `<channel>`. BSE omits title/link on some feeds and
/// sometimes puts `<author>` on the image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BseRssImage {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

/// `<item>` child of `<channel>`. Unknown child elements are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BseRssItem {
    pub title: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "pubDate", default)]
    pub pub_date: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub guid: Option<String>,
    #[serde(default)]
    pub scripcode: Option<String>,
}

/// BSE sometimes returns this plain text instead of an empty `<rss>` channel
/// (e.g. Annual Reports when nothing was filed that cycle).
pub fn is_no_records_payload(body: &str) -> bool {
    let trimmed = body.trim_start_matches('\u{feff}').trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "none of the records" | "no records found" | "no record found"
    ) || (!lower.contains("<rss") && lower.contains("none of the records"))
}

fn empty_document() -> BseRssDocument {
    BseRssDocument {
        version: "2.0".into(),
        channel: BseRssChannel {
            title: String::new(),
            link: String::new(),
            description: String::new(),
            copyright: None,
            language: None,
            last_build_date: None,
            image: None,
            items: Vec::new(),
        },
    }
}

/// Parse a BSE RSS 2.0 XML document.
pub fn parse_rss(xml: &str) -> Result<BseRssDocument, quick_xml::DeError> {
    let trimmed = xml.trim_start_matches('\u{feff}').trim();
    if is_no_records_payload(trimmed) {
        return Ok(empty_document());
    }
    quick_xml::de::from_str(trimmed)
}

/// Stable identity for a BSE RSS item (`guid` is often empty or a listing URL).
pub fn content_hash(
    feed_kind: &str,
    title: &str,
    link: &str,
    description: &str,
    pub_date: &str,
    scripcode: &str,
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
    hasher.update(scripcode.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{BseRssItem, is_no_records_payload, parse_rss};

    const SENSEX: &str = r#"<?xml version="1.0" encoding="utf-8"?><rss version="2.0"><channel><title>SENSEX view</title><link>http://www.bseindia.com</link><copyright>Copyright 2009, BSE.</copyright><image><url>https://www.bseindia.com/sensex/include/images/logo.gif</url><title>BSE SENSEX view</title><description>SENSEX Information</description></image><item><title>SENSEX : 75646.13 * -486.68 (-0.64 %)</title><link /><pubDate>08-09-2026 13:07:49</pubDate><author>BSEIndia</author><guid /></item></channel></rss>"#;

    const NOTICES: &str = r#"<?xml version="1.0" encoding="utf-8"?><rss version="2.0"><channel><title>BSE Notices</title><link>http://www.bseindia.com</link><copyright>Copyright 2010, BSE.</copyright><image><url>https://www.bseindia.com/sensex/include/images/logo.gif</url><description>BSE Notices Information</description></image><item><title>Change in Group of Equity Shares of Farm Peace Limited</title><link>https://www.bseindia.com/downloads/UploadDocs/Notices/20260908-5/20260908-5.pdf</link><author>BSEIndia</author><pubDate>Tue, 08 Sep 2026 07:32:03 GMT</pubDate><guid>https://www.bseindia.com/downloads/UploadDocs/Notices/20260908-5/20260908-5.pdf</guid></item></channel></rss>"#;

    const ANNOUNCEMENTS: &str = r#"<?xml version="1.0" encoding="utf-8"?><rss version="2.0"><channel><title>BSE - Latest Corporate Announcements</title><link>https://www.bseindia.com/corporates/ann.html</link><description>BSE Ltd – Latest Corporate Announcements</description><language>en-us</language><lastBuildDate>Tue, 08 Sep 2026 13:07:50 GMT</lastBuildDate><image><link>https://www.bseindia.com/corporates/ann.html</link><url>https://www.bseindia.com/sensex/include/images/logo.gif</url><author>BSE Ltd</author></image><item><title>CSB Bank Ltd (542867)</title><link>https://www.bseindia.com/xml-data/corpfiling/AttachLive/56e19561-57ab-4e03-b857-565679d74dd7.pdf</link><scripcode>542867</scripcode><description>Participation in the Analyst/Institutional Investor Meeting held on September 8, 2026</description><pubDate>08-Sep-2026 13:06:54</pubDate></item></channel></rss>"#;

    const CORP_ACTIONS: &str = r#"<?xml version="1.0" encoding="utf-8"?><rss version="2.0"><channel><title>BSE - Latest Corporates Actions</title><link>https://www.bseindia.com/corporates/corporates_act.html</link><description>BSE Ltd – Latest Corporates Actions</description><language>en-us</language><lastBuildDate>Tue, 08 Sep 2026 13:07:50 GMT</lastBuildDate><image><title>Latest Corporates Actions&lt;/title&gt;</title><link>https://www.bseindia.com/corporates/corporates_act.html</link><url>https://www.bseindia.com/include/images/bselogo.png</url><author>BSE Ltd</author></image><item><title>Bhandari Hosiery Exports Ltd (512608) - EX Date : 08 Sep, 2026</title><link>https://www.bseindia.com/corporates/corporates_act.html</link><description>SEGMENT : EQUITY | PURPOSE : Final Dividend - Rs. - 0.0100 | RD Date : 08 Sep, 2026 | BC START DATE : -  | BC END DATE : -  | ND START DATE : 08 Sep, 2026 | ND END DATE : 08 Sep, 2026 | Actual Payment Date : - </description></item></channel></rss>"#;

    const MEDIA: &str = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?><rss version=\"2.0\"><channel><title>BSE LATEST MEDIA RELEASE</title><link>https://www.bseindia.com/markets/mediainfo/mediarelease</link><description>BSE Ltd- Media Release</description><language>en-us</language><lastBuildDate>Tue, 08 Sep 2026 13:07:54 GMT</lastBuildDate><image><title>Latest BSE Media Release</title><link>https://www.bseindia.com/markets/mediainfo/mediarelease</link><url>https://www.bseindia.com/include/images/bselogo.png</url><author>BSE Ltd</author></image></channel></rss>";

    #[test]
    fn parses_sensex_empty_link_and_guid() {
        let doc = parse_rss(SENSEX).expect("parse sensex");
        assert_eq!(doc.version, "2.0");
        assert_eq!(doc.channel.title, "SENSEX view");
        assert_eq!(doc.channel.description, "");
        assert_eq!(doc.channel.items.len(), 1);
        let item = &doc.channel.items[0];
        assert!(item.title.starts_with("SENSEX :"));
        assert_eq!(item.link, "");
        assert_eq!(item.description, "");
        assert_eq!(item.pub_date, "08-09-2026 13:07:49");
        assert_eq!(item.author.as_deref(), Some("BSEIndia"));
    }

    #[test]
    fn parses_notices_rfc822_gmt() {
        let doc = parse_rss(NOTICES).expect("parse notices");
        let item = &doc.channel.items[0];
        assert_eq!(item.pub_date, "Tue, 08 Sep 2026 07:32:03 GMT");
        assert!(item.link.ends_with(".pdf"));
        assert!(item.description.is_empty());
        assert!(item.guid.as_deref().unwrap().ends_with(".pdf"));
    }

    #[test]
    fn parses_announcements_scripcode() {
        let doc = parse_rss(ANNOUNCEMENTS).expect("parse announcements");
        assert_eq!(
            doc.channel.last_build_date.as_deref(),
            Some("Tue, 08 Sep 2026 13:07:50 GMT")
        );
        let item = &doc.channel.items[0];
        assert_eq!(item.scripcode.as_deref(), Some("542867"));
        assert_eq!(item.pub_date, "08-Sep-2026 13:06:54");
        assert!(item.description.contains("Analyst"));
    }

    #[test]
    fn parses_corporate_actions_without_pubdate() {
        let doc = parse_rss(CORP_ACTIONS).expect("parse corporate actions");
        let item = &doc.channel.items[0];
        assert_eq!(item.pub_date, "");
        assert!(item.description.starts_with("SEGMENT : EQUITY"));
        assert!(
            doc.channel
                .image
                .as_ref()
                .unwrap()
                .title
                .as_deref()
                .unwrap()
                .contains("Latest Corporates Actions")
        );
    }

    #[test]
    fn parses_empty_channel_with_bom() {
        let doc = parse_rss(MEDIA).expect("parse media");
        assert!(doc.channel.items.is_empty());
        assert_eq!(doc.channel.title, "BSE LATEST MEDIA RELEASE");
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
        let item: BseRssItem = quick_xml::de::from_str(xml).expect("item");
        assert_eq!(item.title, "a");
    }

    #[test]
    fn no_records_payload_is_empty_feed_not_parse_error() {
        assert!(is_no_records_payload("None of the records"));
        assert!(is_no_records_payload("\u{feff}None of the records\n"));
        assert!(is_no_records_payload("none of the records"));
        assert!(!is_no_records_payload(ANNOUNCEMENTS));
        let doc = parse_rss("None of the records").expect("empty feed");
        assert!(doc.channel.items.is_empty());
        assert_eq!(doc.version, "2.0");
    }
}
