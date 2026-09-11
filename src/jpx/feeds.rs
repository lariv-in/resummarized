//! Closed catalog of official JPX English RSS endpoints from
//! https://www.jpx.co.jp/english/rss/index.html.

/// Discriminator for one of the six official JPX English RSS feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum JpxFeedKind {
    MarketNews,
    NewsRelease,
    EquitiesHalt,
    DerivativesHalt,
    Alerts,
    SiteUpdates,
}

impl JpxFeedKind {
    pub const ALL: &[JpxFeedKind] = &[
        JpxFeedKind::MarketNews,
        JpxFeedKind::NewsRelease,
        JpxFeedKind::EquitiesHalt,
        JpxFeedKind::DerivativesHalt,
        JpxFeedKind::Alerts,
        JpxFeedKind::SiteUpdates,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            JpxFeedKind::MarketNews => "market-news",
            JpxFeedKind::NewsRelease => "news-release",
            JpxFeedKind::EquitiesHalt => "equities-halt",
            JpxFeedKind::DerivativesHalt => "derivatives-halt",
            JpxFeedKind::Alerts => "alerts",
            JpxFeedKind::SiteUpdates => "site-updates",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            JpxFeedKind::MarketNews => "Market News",
            JpxFeedKind::NewsRelease => "News Release",
            JpxFeedKind::EquitiesHalt => "Trading Halt (Equities)",
            JpxFeedKind::DerivativesHalt => "Trading Halt (Derivatives)",
            JpxFeedKind::Alerts => "Alerts on Unclear Information",
            JpxFeedKind::SiteUpdates => "Site Updates",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            JpxFeedKind::MarketNews => "https://www.jpx.co.jp/english/rss/markets_news.xml",
            JpxFeedKind::NewsRelease => "https://www.jpx.co.jp/english/rss/jpx-news.xml",
            JpxFeedKind::EquitiesHalt => "https://www.jpx.co.jp/english/rss/equities-suspended.xml",
            JpxFeedKind::DerivativesHalt => {
                "https://www.jpx.co.jp/english/rss/derivatives-suspended.xml"
            }
            JpxFeedKind::Alerts => "https://www.jpx.co.jp/english/rss/alerts.xml",
            JpxFeedKind::SiteUpdates => "https://www.jpx.co.jp/english/rss/site-updates.xml",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.slug() == slug)
    }

    /// JPX items typically omit `<description>`, so list tables hide that column.
    pub fn shows_description_column(self) -> bool {
        false
    }
}

/// Per-item document URL, if the RSS `<link>` is a real item page.
pub fn item_document_link(url: &str) -> Option<&str> {
    let url = url.trim();
    if url.is_empty()
        || url.starts_with("https://www.jpx.co.jp/english/rss/") && url.ends_with(".xml")
    {
        return None;
    }
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::{JpxFeedKind, item_document_link};
    use std::collections::HashSet;

    #[test]
    fn catalog_has_unique_slugs_and_urls() {
        let slugs: HashSet<_> = JpxFeedKind::ALL.iter().map(|k| k.slug()).collect();
        let urls: HashSet<_> = JpxFeedKind::ALL.iter().map(|k| k.url()).collect();
        assert_eq!(slugs.len(), JpxFeedKind::ALL.len());
        assert_eq!(urls.len(), JpxFeedKind::ALL.len());
        assert_eq!(JpxFeedKind::ALL.len(), 6);
        assert_eq!(JpxFeedKind::ALL[0], JpxFeedKind::MarketNews);
    }

    #[test]
    fn from_slug_round_trips() {
        for kind in JpxFeedKind::ALL {
            assert_eq!(JpxFeedKind::from_slug(kind.slug()), Some(*kind));
        }
        assert_eq!(JpxFeedKind::from_slug("nope"), None);
    }

    #[test]
    fn description_column_is_hidden() {
        for kind in JpxFeedKind::ALL {
            assert!(!kind.shows_description_column());
        }
    }

    #[test]
    fn item_document_link_skips_feed_xml_and_empty() {
        assert_eq!(item_document_link(""), None);
        assert_eq!(
            item_document_link("https://www.jpx.co.jp/english/rss/markets_news.xml"),
            None
        );
        assert_eq!(
            item_document_link(
                "https://www.jpx.co.jp/english/equities/products/tpm/issues/index.html"
            ),
            Some("https://www.jpx.co.jp/english/equities/products/tpm/issues/index.html")
        );
    }
}
