//! Closed catalog of official Euronext Athens RSS endpoints from
//! https://athens.euronext.com/en/rss.

/// Extra typed columns shown on issuer-like feed lists.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EuronextExtraField {
    Ticker,
    Company,
}

impl EuronextExtraField {
    pub fn sort_key(self) -> &'static str {
        match self {
            Self::Ticker => "Ticker",
            Self::Company => "Company",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ticker => "Ticker",
            Self::Company => "Company",
        }
    }
}

const COMPANY_FIELDS: &[EuronextExtraField] =
    &[EuronextExtraField::Ticker, EuronextExtraField::Company];

/// Discriminator for one of the ten official Euronext Athens RSS feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EuronextFeedKind {
    FullFeed,
    PressReleases,
    IssuerAnnouncements,
    CorporateActions,
    ForcedSales,
    SecuritiesMarketBulletin,
    RiskManagement,
    FinancialData,
    MarketNotices,
    NonListed,
}

impl EuronextFeedKind {
    pub const ALL: &[EuronextFeedKind] = &[
        EuronextFeedKind::FullFeed,
        EuronextFeedKind::PressReleases,
        EuronextFeedKind::IssuerAnnouncements,
        EuronextFeedKind::CorporateActions,
        EuronextFeedKind::ForcedSales,
        EuronextFeedKind::SecuritiesMarketBulletin,
        EuronextFeedKind::RiskManagement,
        EuronextFeedKind::FinancialData,
        EuronextFeedKind::MarketNotices,
        EuronextFeedKind::NonListed,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            EuronextFeedKind::FullFeed => "full-feed",
            EuronextFeedKind::PressReleases => "press-releases",
            EuronextFeedKind::IssuerAnnouncements => "issuer-announcements",
            EuronextFeedKind::CorporateActions => "corporate-actions",
            EuronextFeedKind::ForcedSales => "forced-sales",
            EuronextFeedKind::SecuritiesMarketBulletin => "securities-market-bulletin",
            EuronextFeedKind::RiskManagement => "risk-management",
            EuronextFeedKind::FinancialData => "financial-data",
            EuronextFeedKind::MarketNotices => "market-notices",
            EuronextFeedKind::NonListed => "non-listed",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            EuronextFeedKind::FullFeed => "Full Feed",
            EuronextFeedKind::PressReleases => "Press Releases",
            EuronextFeedKind::IssuerAnnouncements => "Issuer Announcements",
            EuronextFeedKind::CorporateActions => "Corporate Actions",
            EuronextFeedKind::ForcedSales => "Forced Sales",
            EuronextFeedKind::SecuritiesMarketBulletin => "Securities Market Bulletin",
            EuronextFeedKind::RiskManagement => "Risk Management",
            EuronextFeedKind::FinancialData => "Financial Data",
            EuronextFeedKind::MarketNotices => "Market Notices",
            EuronextFeedKind::NonListed => "Non-Listed Issuers",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            EuronextFeedKind::FullFeed => "https://athens.euronext.com/en/rss/full-feed",
            EuronextFeedKind::PressReleases => "https://athens.euronext.com/en/rss/press-releases",
            EuronextFeedKind::IssuerAnnouncements => {
                "https://athens.euronext.com/en/rss/issuer-announcements"
            }
            EuronextFeedKind::CorporateActions => {
                "https://athens.euronext.com/en/rss/corporate-actions"
            }
            EuronextFeedKind::ForcedSales => "https://athens.euronext.com/en/rss/forced-sales",
            EuronextFeedKind::SecuritiesMarketBulletin => {
                "https://athens.euronext.com/en/rss/securities-market-bulletin"
            }
            EuronextFeedKind::RiskManagement => {
                "https://athens.euronext.com/en/rss/risk-management"
            }
            EuronextFeedKind::FinancialData => "https://athens.euronext.com/en/rss/financial-data",
            EuronextFeedKind::MarketNotices => "https://athens.euronext.com/en/rss/market-notices",
            EuronextFeedKind::NonListed => "https://athens.euronext.com/en/rss/non-listed",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.slug() == slug)
    }

    pub fn extra_list_fields(self) -> &'static [EuronextExtraField] {
        if self.shows_company_columns() {
            COMPANY_FIELDS
        } else {
            &[]
        }
    }

    pub fn shows_description_column(self) -> bool {
        true
    }

    pub fn shows_company_columns(self) -> bool {
        matches!(
            self,
            EuronextFeedKind::FullFeed
                | EuronextFeedKind::IssuerAnnouncements
                | EuronextFeedKind::CorporateActions
                | EuronextFeedKind::FinancialData
                | EuronextFeedKind::ForcedSales
                | EuronextFeedKind::NonListed
        )
    }
}

/// Per-item document URL, if the RSS `<link>` is a real item page.
pub fn item_document_link(url: &str) -> Option<&str> {
    let url = url.trim();
    if url.is_empty()
        || url == "https://athens.euronext.com/"
        || url == "https://athens.euronext.com/en"
        || url == "https://athens.euronext.com"
    {
        return None;
    }
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::{EuronextFeedKind, item_document_link};
    use std::collections::HashSet;

    #[test]
    fn catalog_has_unique_slugs_and_urls() {
        let slugs: HashSet<_> = EuronextFeedKind::ALL.iter().map(|k| k.slug()).collect();
        let urls: HashSet<_> = EuronextFeedKind::ALL.iter().map(|k| k.url()).collect();
        assert_eq!(slugs.len(), EuronextFeedKind::ALL.len());
        assert_eq!(urls.len(), EuronextFeedKind::ALL.len());
        assert_eq!(EuronextFeedKind::ALL.len(), 10);
        assert_eq!(EuronextFeedKind::ALL[0], EuronextFeedKind::FullFeed);
    }

    #[test]
    fn from_slug_round_trips() {
        for kind in EuronextFeedKind::ALL {
            assert_eq!(EuronextFeedKind::from_slug(kind.slug()), Some(*kind));
        }
        assert_eq!(EuronextFeedKind::from_slug("nope"), None);
    }

    #[test]
    fn issuer_like_feeds_show_company_columns() {
        assert!(EuronextFeedKind::IssuerAnnouncements.shows_company_columns());
        assert!(!EuronextFeedKind::PressReleases.shows_company_columns());
        assert!(!EuronextFeedKind::RiskManagement.shows_company_columns());
        assert_eq!(EuronextFeedKind::FullFeed.extra_list_fields().len(), 2);
    }

    #[test]
    fn item_document_link_skips_channel_home() {
        assert_eq!(item_document_link(""), None);
        assert_eq!(item_document_link("https://athens.euronext.com/en"), None);
        assert_eq!(
            item_document_link("https://athens.euronext.com/en/node/969219"),
            Some("https://athens.euronext.com/en/node/969219")
        );
    }
}
