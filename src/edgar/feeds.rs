//! Catalog of SEC EDGAR Atom endpoints.

/// Extra typed column shown on EDGAR feed lists.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EdgarExtraField {
    Category,
}

impl EdgarExtraField {
    pub fn sort_key(self) -> &'static str {
        match self {
            Self::Category => "Category",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Category => "Form Type",
        }
    }
}

/// Discriminator for one of the SEC EDGAR Atom feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EdgarFeedKind {
    EightK,
    TenK,
    AllFilings,
    Form4,
}

impl EdgarFeedKind {
    pub const ALL: &[EdgarFeedKind] = &[
        EdgarFeedKind::EightK,
        EdgarFeedKind::TenK,
        EdgarFeedKind::AllFilings,
        EdgarFeedKind::Form4,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            EdgarFeedKind::EightK => "8-k",
            EdgarFeedKind::TenK => "10-k-10-q",
            EdgarFeedKind::AllFilings => "all-filings",
            EdgarFeedKind::Form4 => "form-4",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            EdgarFeedKind::EightK => "8-K Material Events",
            EdgarFeedKind::TenK => "10-K / 10-Q Periodic Reports",
            EdgarFeedKind::AllFilings => "All SEC Filings",
            EdgarFeedKind::Form4 => "Form 4 Insider Transactions",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            EdgarFeedKind::EightK => {
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001120193&type=8-K&owner=exclude&count=15&output=atom"
            }
            EdgarFeedKind::TenK => {
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001120193&type=10&owner=exclude&count=15&output=atom"
            }
            EdgarFeedKind::AllFilings => {
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001120193&owner=include&count=15&output=atom"
            }
            EdgarFeedKind::Form4 => {
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK=0001120193&type=4&owner=only&count=15&output=atom"
            }
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.slug() == slug)
    }

    pub fn extra_list_fields(self) -> &'static [EdgarExtraField] {
        &[EdgarExtraField::Category]
    }

    pub fn shows_description_column(self) -> bool {
        true
    }

    pub fn stores_form_category(self) -> bool {
        true
    }
}

/// SEC titles are e.g. `144  - Report of proposed sale of securities` or `8-K: Report...`.
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

/// Per-item document URL, if the link is a valid SEC URL.
pub fn item_document_link(url: &str) -> Option<&str> {
    let url = url.trim();
    if url.is_empty() || url == "https://www.sec.gov/" {
        return None;
    }
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::{EdgarFeedKind, form_type_from_title, item_document_link};
    use std::collections::HashSet;

    #[test]
    fn catalog_has_unique_slugs_and_urls() {
        let slugs: HashSet<_> = EdgarFeedKind::ALL.iter().map(|k| k.slug()).collect();
        let urls: HashSet<_> = EdgarFeedKind::ALL.iter().map(|k| k.url()).collect();
        assert_eq!(slugs.len(), EdgarFeedKind::ALL.len());
        assert_eq!(urls.len(), EdgarFeedKind::ALL.len());
        assert_eq!(EdgarFeedKind::ALL.len(), 4);
    }

    #[test]
    fn from_slug_round_trips() {
        for kind in EdgarFeedKind::ALL {
            assert_eq!(EdgarFeedKind::from_slug(kind.slug()), Some(*kind));
        }
        assert_eq!(EdgarFeedKind::from_slug("nope"), None);
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
            form_type_from_title("144  - Report of proposed sale of securities"),
            Some("144".into())
        );
        assert_eq!(
            form_type_from_title("Annual Report"),
            None
        );
    }

    #[test]
    fn item_document_link_skips_sec_home() {
        assert_eq!(item_document_link(""), None);
        assert_eq!(item_document_link("https://www.sec.gov/"), None);
        assert_eq!(
            item_document_link(
                "https://www.sec.gov/Archives/edgar/data/1120193/000195004726009284/index.htm"
            ),
            Some("https://www.sec.gov/Archives/edgar/data/1120193/000195004726009284/index.htm")
        );
    }
}
