//! EDGAR Atom 1.0 types and parser.

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgarAtomItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: String,
    pub guid: Option<String>,
    pub category: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgarParsedItems {
    pub last_build_date: Option<String>,
    pub items: Vec<EdgarAtomItem>,
}

#[derive(Debug, Deserialize)]
struct AtomFeed {
    #[serde(default)]
    updated: Option<String>,
    #[serde(rename = "entry", default)]
    entries: Vec<AtomEntry>,
}

#[derive(Debug, Deserialize)]
struct AtomEntry {
    #[serde(default)]
    title: String,
    #[serde(default)]
    link: AtomLink,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    updated: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    category: Option<AtomCategory>,
    #[serde(default)]
    author: Option<AtomAuthor>,
}

#[derive(Debug, Default, Deserialize)]
struct AtomLink {
    #[serde(rename = "@href", default)]
    href: String,
}

#[derive(Debug, Default, Deserialize)]
struct AtomCategory {
    #[serde(rename = "@term", default)]
    term: String,
}

#[derive(Debug, Default, Deserialize)]
struct AtomAuthor {
    #[serde(default)]
    name: Option<String>,
}

/// Parse an EDGAR Atom 1.0 document.
pub fn parse_atom(xml: &str) -> Result<EdgarParsedItems, quick_xml::DeError> {
    let xml = xml.trim_start_matches('\u{feff}').trim();
    let feed: AtomFeed = quick_xml::de::from_str(xml)?;
    let items = feed
        .entries
        .into_iter()
        .map(|entry| {
            let category = {
                let term = entry.category.as_ref().map(|c| c.term.trim()).unwrap_or("");
                (!term.is_empty()).then(|| term.to_string())
            };
            let guid = entry.id.filter(|s| !s.trim().is_empty());
            let author = entry
                .author
                .and_then(|a| a.name)
                .filter(|s| !s.trim().is_empty());
            EdgarAtomItem {
                title: entry.title,
                link: entry.link.href,
                description: entry.summary.unwrap_or_default(),
                pub_date: entry.updated,
                guid,
                category,
                author,
            }
        })
        .collect();
    Ok(EdgarParsedItems {
        last_build_date: feed.updated,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_atom;

    const EDGAR: &str = r#"<?xml version="1.0" encoding="ISO-8859-1" ?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>EDGAR Filer NASDAQ, INC.</title>
  <updated>2026-09-10T16:15:00-04:00</updated>
  <entry>
    <category label="form type" scheme="https://www.sec.gov/" term="144" />
    <id>urn:tag:sec.gov,2008:accession-number=0001950047-26-009284</id>
    <link href="https://www.sec.gov/Archives/edgar/data/1120193/000195004726009284/0001950047-26-009284-index.htm" rel="alternate" type="text/html" />
    <summary type="html"> &lt;b&gt;Filed:&lt;/b&gt; 2026-09-10</summary>
    <title>144  - Report of proposed sale of securities</title>
    <updated>2026-09-10T16:12:55-04:00</updated>
  </entry>
</feed>"#;

    #[test]
    fn parses_edgar_atom_entry() {
        let parsed = parse_atom(EDGAR).expect("parse atom");
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(
            parsed.last_build_date.as_deref(),
            Some("2026-09-10T16:15:00-04:00")
        );
        let item = &parsed.items[0];
        assert!(item.title.starts_with("144"));
        assert!(item.link.contains("000195004726009284"));
        assert_eq!(item.category.as_deref(), Some("144"));
        assert_eq!(item.pub_date, "2026-09-10T16:12:55-04:00");
        assert!(
            item.guid
                .as_ref()
                .unwrap()
                .contains("0001950047-26-009284")
        );
    }
}
