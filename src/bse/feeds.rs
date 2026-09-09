//! Closed catalog of official BSE RSS endpoints from https://beta.bseindia.com/rss-feed.html.

use super::description::DescriptionField;

/// Discriminator for one of the 12 official BSE RSS feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BseFeedKind {
    Sensex,
    Notices,
    Announcements,
    CorporateActions,
    VotingResults,
    AnnualReports,
    BoardMeetings,
    InsiderTrading,
    FinancialResults,
    ShareholdingPattern,
    MediaRelease,
    IndexMediaRelease,
}

impl BseFeedKind {
    pub const ALL: &[BseFeedKind] = &[
        BseFeedKind::Sensex,
        BseFeedKind::Notices,
        BseFeedKind::Announcements,
        BseFeedKind::CorporateActions,
        BseFeedKind::VotingResults,
        BseFeedKind::AnnualReports,
        BseFeedKind::BoardMeetings,
        BseFeedKind::InsiderTrading,
        BseFeedKind::FinancialResults,
        BseFeedKind::ShareholdingPattern,
        BseFeedKind::MediaRelease,
        BseFeedKind::IndexMediaRelease,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            BseFeedKind::Sensex => "sensex",
            BseFeedKind::Notices => "notices",
            BseFeedKind::Announcements => "announcements",
            BseFeedKind::CorporateActions => "corporate-actions",
            BseFeedKind::VotingResults => "voting-results",
            BseFeedKind::AnnualReports => "annual-reports",
            BseFeedKind::BoardMeetings => "board-meetings",
            BseFeedKind::InsiderTrading => "insider-trading",
            BseFeedKind::FinancialResults => "financial-results",
            BseFeedKind::ShareholdingPattern => "shareholding-pattern",
            BseFeedKind::MediaRelease => "media-release",
            BseFeedKind::IndexMediaRelease => "index-media-release",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            BseFeedKind::Sensex => "SENSEX",
            BseFeedKind::Notices => "Notices",
            BseFeedKind::Announcements => "Corporate Announcements",
            BseFeedKind::CorporateActions => "Corporate Actions",
            BseFeedKind::VotingResults => "Voting Results",
            BseFeedKind::AnnualReports => "Annual Reports",
            BseFeedKind::BoardMeetings => "Board Meetings",
            BseFeedKind::InsiderTrading => "Insider Trading",
            BseFeedKind::FinancialResults => "Financial Results",
            BseFeedKind::ShareholdingPattern => "Shareholding Patterns",
            BseFeedKind::MediaRelease => "BSE Media Release",
            BseFeedKind::IndexMediaRelease => "BSE Index Media Release",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            BseFeedKind::Sensex => "https://beta.bseindia.com/data/xml/sensexrss.xml",
            BseFeedKind::Notices => "https://beta.bseindia.com/data/xml/notices.xml",
            BseFeedKind::Announcements => "https://beta.bseindia.com/data/xml/announcements.xml",
            BseFeedKind::CorporateActions => {
                "https://beta.bseindia.com/data/XML/CorpActionFeed.xml"
            }
            BseFeedKind::VotingResults => "https://beta.bseindia.com/data/XML/VotingResultFeed.xml",
            BseFeedKind::AnnualReports => "https://beta.bseindia.com/Data/XML/AnnualReportFeed.xml",
            BseFeedKind::BoardMeetings => {
                "https://beta.bseindia.com/Data/XML/BoardMeetingsFeed.xml"
            }
            BseFeedKind::InsiderTrading => {
                "https://beta.bseindia.com/Data/XML/InsiderTradingFeed.xml"
            }
            BseFeedKind::FinancialResults => {
                "https://beta.bseindia.com/Data/XML/FinancialResultsFeed.xml"
            }
            BseFeedKind::ShareholdingPattern => {
                "https://beta.bseindia.com/Data/XML/ShareholdingPattern_Feed.xml"
            }
            BseFeedKind::MediaRelease => "https://beta.bseindia.com/Data/XML/MediaRelease_Feed.xml",
            BseFeedKind::IndexMediaRelease => {
                "https://beta.bseindia.com/Data/XML/IndexMediaRelease_Feed.xml"
            }
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.slug() == slug)
    }

    /// Description encodings this feed stores as typed columns (list + sort).
    pub fn extra_list_fields(self) -> &'static [DescriptionField] {
        use DescriptionField as F;
        match self {
            BseFeedKind::Announcements => &[F::Scripcode],
            BseFeedKind::AnnualReports => &[F::Scripcode, F::AsOnDate],
            BseFeedKind::BoardMeetings => &[F::Scripcode, F::MeetingDate, F::Purpose],
            BseFeedKind::CorporateActions => &[
                F::Scripcode,
                F::Segment,
                F::Purpose,
                F::RdDate,
                F::BcStartDate,
                F::BcEndDate,
                F::NdStartDate,
                F::NdEndDate,
                F::ActualPaymentDate,
            ],
            BseFeedKind::FinancialResults => &[
                F::Scripcode,
                F::AuditedUnaudited,
                F::StandaloneConsolidated,
                F::PeriodStartDate,
                F::PeriodEndDate,
                F::IndAs,
            ],
            BseFeedKind::InsiderTrading => &[F::Scripcode, F::TypeOfSecurity],
            BseFeedKind::ShareholdingPattern => &[
                F::Scripcode,
                F::AsOnDate,
                F::PromoterAndGroup,
                F::PublicVal,
                F::Status,
                F::SubmissionDate,
                F::RevisedFilingDate,
            ],
            BseFeedKind::VotingResults => &[F::Scripcode, F::MeetingDate, F::MeetingType],
            BseFeedKind::Sensex
            | BseFeedKind::Notices
            | BseFeedKind::MediaRelease
            | BseFeedKind::IndexMediaRelease => &[],
        }
    }

    pub fn shows_description_column(self) -> bool {
        matches!(
            self,
            BseFeedKind::Announcements | BseFeedKind::MediaRelease | BseFeedKind::IndexMediaRelease
        )
    }
}

/// Turn a BSE RSS `<link>` into an absolute URL. Relative paths are hosted on www.
pub fn normalize_item_link(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return String::new();
    }
    if url.starts_with('/') {
        format!("https://www.bseindia.com{url}")
    } else {
        url.to_string()
    }
}

/// Per-item filing URL, if the RSS `<link>` is not a channel listing page.
pub fn item_document_link(url: &str) -> Option<&str> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let listing = path.contains("/corporates/corporates_act.html")
        || path.contains("/corporates/board_meeting.aspx")
        || path.contains("/corporates/Insider_Trading_new.aspx");
    (!listing).then_some(url)
}

/// BSE titles commonly end with `(scripcode)`, e.g. `CSB Bank Ltd (542867)`.
pub fn scripcode_from_title(title: &str) -> Option<String> {
    let bytes = title.as_bytes();
    let mut i = bytes.len();
    while i > 0 {
        if bytes[i - 1] == b')' {
            let end = i - 1;
            let mut start = end;
            while start > 0 && bytes[start - 1].is_ascii_digit() {
                start -= 1;
            }
            let digits = &title[start..end];
            if (5..=6).contains(&digits.len()) && start > 0 && bytes[start - 1] == b'(' {
                return Some(digits.to_string());
            }
        }
        i -= 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{BseFeedKind, item_document_link, normalize_item_link, scripcode_from_title};
    use std::collections::HashSet;

    #[test]
    fn catalog_has_unique_slugs_and_urls() {
        let slugs: HashSet<_> = BseFeedKind::ALL.iter().map(|k| k.slug()).collect();
        let urls: HashSet<_> = BseFeedKind::ALL.iter().map(|k| k.url()).collect();
        assert_eq!(slugs.len(), BseFeedKind::ALL.len());
        assert_eq!(urls.len(), BseFeedKind::ALL.len());
        assert_eq!(BseFeedKind::ALL.len(), 12);
    }

    #[test]
    fn from_slug_round_trips() {
        for kind in BseFeedKind::ALL {
            assert_eq!(BseFeedKind::from_slug(kind.slug()), Some(*kind));
        }
        assert_eq!(BseFeedKind::from_slug("nope"), None);
    }

    #[test]
    fn normalize_item_link_prefixes_relative_paths() {
        assert_eq!(normalize_item_link(""), "");
        assert_eq!(normalize_item_link("   "), "");
        assert_eq!(
            normalize_item_link("/XBRLFILES/SHPXBRLDataXML/517415_SP.html"),
            "https://www.bseindia.com/XBRLFILES/SHPXBRLDataXML/517415_SP.html"
        );
        assert_eq!(
            normalize_item_link("https://www.bseindia.com/downloads/a.pdf"),
            "https://www.bseindia.com/downloads/a.pdf"
        );
    }

    #[test]
    fn item_document_link_skips_listing_pages_and_blanks() {
        assert_eq!(item_document_link(""), None);
        assert_eq!(
            item_document_link("https://www.bseindia.com/corporates/corporates_act.html"),
            None
        );
        assert_eq!(
            item_document_link("https://www.bseindia.com/corporates/board_meeting.aspx"),
            None
        );
        assert_eq!(
            item_document_link("https://www.bseindia.com/xml-data/corpfiling/AttachLive/a.pdf"),
            Some("https://www.bseindia.com/xml-data/corpfiling/AttachLive/a.pdf")
        );
    }

    #[test]
    fn scripcode_from_title_reads_trailing_code_not_percent() {
        assert_eq!(
            scripcode_from_title("CSB Bank Ltd (542867)"),
            Some("542867".into())
        );
        assert_eq!(
            scripcode_from_title("Bhandari Hosiery Exports Ltd (512608) - EX Date : 08 Sep, 2026"),
            Some("512608".into())
        );
        assert_eq!(
            scripcode_from_title("ZF Steering Gear India Ltd-$ (505163)"),
            Some("505163".into())
        );
        assert_eq!(
            scripcode_from_title("SENSEX : 75646.13 * -486.68 (-0.64 %)"),
            None
        );
        assert_eq!(
            scripcode_from_title("Change in Group of Equity Shares"),
            None
        );
    }
}
