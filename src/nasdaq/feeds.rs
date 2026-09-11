//! Closed catalog of official Nasdaq, Inc. IR RSS endpoints from
//! https://ir.nasdaq.com/tools/rss-feeds.

/// Extra typed column shown on SEC feed lists.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NasdaqExtraField {
    Category,
}

impl NasdaqExtraField {
    pub fn sort_key(self) -> &'static str {
        match self {
            Self::Category => "Category",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Category => "Category",
        }
    }
}

/// Discriminator for one of the four official Nasdaq IR RSS feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum NasdaqFeedKind {
    NewsReleases,
    FinancialReleases,
    SecFilings,
    Form4SecFilings,
}

impl NasdaqFeedKind {
    pub const ALL: &[NasdaqFeedKind] = &[
        NasdaqFeedKind::NewsReleases,
        NasdaqFeedKind::FinancialReleases,
        NasdaqFeedKind::SecFilings,
        NasdaqFeedKind::Form4SecFilings,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            NasdaqFeedKind::NewsReleases => "news-releases",
            NasdaqFeedKind::FinancialReleases => "financial-releases",
            NasdaqFeedKind::SecFilings => "sec-filings",
            NasdaqFeedKind::Form4SecFilings => "form-4-sec-filings",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            NasdaqFeedKind::NewsReleases => "All News Releases",
            NasdaqFeedKind::FinancialReleases => "Financial Releases",
            NasdaqFeedKind::SecFilings => "All SEC Filings",
            NasdaqFeedKind::Form4SecFilings => "Form 4 SEC Filings",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            NasdaqFeedKind::NewsReleases => "https://ir.nasdaq.com/rss/news-releases.xml?items=15",
            NasdaqFeedKind::FinancialReleases => {
                "https://ir.nasdaq.com/rss/news-releases.xml?items=15&category=Financial"
            }
            NasdaqFeedKind::SecFilings => "https://ir.nasdaq.com/rss/sec-filings.xml?items=15",
            NasdaqFeedKind::Form4SecFilings => {
                "https://ir.nasdaq.com/rss/sec-filings.xml?items=15&sub_group=4"
            }
        }
    }

    /// SEC EDGAR Atom if the official IR RSS endpoint is unreachable.
    pub fn fallback_url(self) -> &'static str {
        match self {
            NasdaqFeedKind::NewsReleases => {
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001120193&type=8-K&owner=exclude&count=15&output=atom"
            }
            NasdaqFeedKind::FinancialReleases => {
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001120193&type=10&owner=exclude&count=15&output=atom"
            }
            NasdaqFeedKind::SecFilings => {
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001120193&owner=include&count=15&output=atom"
            }
            NasdaqFeedKind::Form4SecFilings => {
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001120193&type=4&owner=only&count=15&output=atom"
            }
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.slug() == slug)
    }

    pub fn extra_list_fields(self) -> &'static [NasdaqExtraField] {
        match self {
            NasdaqFeedKind::SecFilings | NasdaqFeedKind::Form4SecFilings => {
                &[NasdaqExtraField::Category]
            }
            NasdaqFeedKind::NewsReleases | NasdaqFeedKind::FinancialReleases => &[],
        }
    }

    pub fn shows_description_column(self) -> bool {
        true
    }

    pub fn stores_form_category(self) -> bool {
        matches!(
            self,
            NasdaqFeedKind::SecFilings | NasdaqFeedKind::Form4SecFilings
        )
    }
}

/// SEC titles are `8-K: Report...` (IR) or `144  - Report...` (EDGAR Atom).
pub fn form_type_from_title(title: &str) -> Option<String> {
    let head = title
        .split_once(':')
        .map(|(h, r)| (h, r))
        .or_else(|| title.split_once(" - "))
        .and_then(|(h, rest)| {
            if rest.trim().is_empty() {
                None
            } else {
                Some(h)
            }
        })?;
    let head = head.trim();
    if head.is_empty() || head.len() > 16 {
        return None;
    }
    if !head
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '/' | ' '))
    {
        return None;
    }
    Some(head.to_string())
}

/// Per-item document URL, if the RSS `<link>` is a real item page.
pub fn item_document_link(url: &str) -> Option<&str> {
    let url = url.trim();
    if url.is_empty() || url == "https://ir.nasdaq.com/" {
        return None;
    }
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::{NasdaqFeedKind, form_type_from_title, item_document_link};
    use std::collections::HashSet;

    #[test]
    fn catalog_has_unique_slugs_and_urls() {
        let slugs: HashSet<_> = NasdaqFeedKind::ALL.iter().map(|k| k.slug()).collect();
        let urls: HashSet<_> = NasdaqFeedKind::ALL.iter().map(|k| k.url()).collect();
        let fallbacks: HashSet<_> = NasdaqFeedKind::ALL
            .iter()
            .map(|k| k.fallback_url())
            .collect();
        assert_eq!(slugs.len(), NasdaqFeedKind::ALL.len());
        assert_eq!(urls.len(), NasdaqFeedKind::ALL.len());
        assert_eq!(fallbacks.len(), NasdaqFeedKind::ALL.len());
        assert_eq!(NasdaqFeedKind::ALL.len(), 4);
    }

    #[test]
    fn from_slug_round_trips() {
        for kind in NasdaqFeedKind::ALL {
            assert_eq!(NasdaqFeedKind::from_slug(kind.slug()), Some(*kind));
        }
        assert_eq!(NasdaqFeedKind::from_slug("nope"), None);
    }

    #[test]
    fn form_type_from_title_reads_sec_prefix() {
        assert_eq!(
            form_type_from_title("8-K: Report of unscheduled material events or corporate event"),
            Some("8-K".into())
        );
        assert_eq!(
            form_type_from_title("4: Statement of changes in beneficial ownership of securities"),
            Some("4".into())
        );
        assert_eq!(
            form_type_from_title(
                "144: Filed by \"insiders\" prior intended sale of restricted stock."
            ),
            Some("144".into())
        );
        assert_eq!(
            form_type_from_title("144  - Report of proposed sale of securities"),
            Some("144".into())
        );
        assert_eq!(
            form_type_from_title("Annual Changes to the Nasdaq-100 Index®"),
            None
        );
    }

    #[test]
    fn item_document_link_skips_channel_home() {
        assert_eq!(item_document_link(""), None);
        assert_eq!(item_document_link("https://ir.nasdaq.com/"), None);
        assert_eq!(
            item_document_link(
                "https://ir.nasdaq.com/sec-filings/sec-filing/8-k/0001193125-25-319238"
            ),
            Some("https://ir.nasdaq.com/sec-filings/sec-filing/8-k/0001193125-25-319238")
        );
    }
}
