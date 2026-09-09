//! Closed catalog of official NSE RSS endpoints from https://www.nseindia.com/static/rss-feed.

use super::description::DescriptionField;

/// Discriminator for one of the 23 official NSE RSS feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum NseFeedKind {
    Announcements,
    AnnualReports,
    BoardMeetings,
    Brsr,
    CorporateActions,
    CorporateGovernance,
    DailyBuyback,
    FinancialResults,
    IntegratedFilingFinancials,
    InsiderTrading,
    InvestorComplaints,
    OfferDocuments,
    RelatedPartyTransactions,
    Regulation29,
    Regulation31,
    ReasonForEncumbrance,
    SecretarialCompliance,
    ShareTransfers,
    ShareholdingPattern,
    StatementOfDeviation,
    UnitholdingPatterns,
    VotingResults,
    Circulars,
}

impl NseFeedKind {
    pub const ALL: &[NseFeedKind] = &[
        NseFeedKind::Announcements,
        NseFeedKind::AnnualReports,
        NseFeedKind::BoardMeetings,
        NseFeedKind::Brsr,
        NseFeedKind::CorporateActions,
        NseFeedKind::CorporateGovernance,
        NseFeedKind::DailyBuyback,
        NseFeedKind::FinancialResults,
        NseFeedKind::IntegratedFilingFinancials,
        NseFeedKind::InsiderTrading,
        NseFeedKind::InvestorComplaints,
        NseFeedKind::OfferDocuments,
        NseFeedKind::RelatedPartyTransactions,
        NseFeedKind::Regulation29,
        NseFeedKind::Regulation31,
        NseFeedKind::ReasonForEncumbrance,
        NseFeedKind::SecretarialCompliance,
        NseFeedKind::ShareTransfers,
        NseFeedKind::ShareholdingPattern,
        NseFeedKind::StatementOfDeviation,
        NseFeedKind::UnitholdingPatterns,
        NseFeedKind::VotingResults,
        NseFeedKind::Circulars,
    ];

    const BASE: &'static str = "https://nsearchives.nseindia.com/content/RSS";

    pub fn slug(self) -> &'static str {
        match self {
            NseFeedKind::Announcements => "announcements",
            NseFeedKind::AnnualReports => "annual-reports",
            NseFeedKind::BoardMeetings => "board-meetings",
            NseFeedKind::Brsr => "brsr",
            NseFeedKind::CorporateActions => "corporate-actions",
            NseFeedKind::CorporateGovernance => "corporate-governance",
            NseFeedKind::DailyBuyback => "daily-buyback",
            NseFeedKind::FinancialResults => "financial-results",
            NseFeedKind::IntegratedFilingFinancials => "integrated-filing-financials",
            NseFeedKind::InsiderTrading => "insider-trading",
            NseFeedKind::InvestorComplaints => "investor-complaints",
            NseFeedKind::OfferDocuments => "offer-documents",
            NseFeedKind::RelatedPartyTransactions => "related-party-transactions",
            NseFeedKind::Regulation29 => "regulation-29",
            NseFeedKind::Regulation31 => "regulation-31",
            NseFeedKind::ReasonForEncumbrance => "reason-for-encumbrance",
            NseFeedKind::SecretarialCompliance => "secretarial-compliance",
            NseFeedKind::ShareTransfers => "share-transfers",
            NseFeedKind::ShareholdingPattern => "shareholding-pattern",
            NseFeedKind::StatementOfDeviation => "statement-of-deviation",
            NseFeedKind::UnitholdingPatterns => "unitholding-patterns",
            NseFeedKind::VotingResults => "voting-results",
            NseFeedKind::Circulars => "circulars",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            NseFeedKind::Announcements => "Announcements",
            NseFeedKind::AnnualReports => "Annual Reports",
            NseFeedKind::BoardMeetings => "Board Meetings",
            NseFeedKind::Brsr => "Business Responsibility and Sustainability Report",
            NseFeedKind::CorporateActions => "Corporate Actions",
            NseFeedKind::CorporateGovernance => "Corporate Governance",
            NseFeedKind::DailyBuyback => "Daily Buy Back / Redemption",
            NseFeedKind::FinancialResults => "Financial Results",
            NseFeedKind::IntegratedFilingFinancials => "Integrated Filing- Financials",
            NseFeedKind::InsiderTrading => "Insider Trading",
            NseFeedKind::InvestorComplaints => "Investor Complaints",
            NseFeedKind::OfferDocuments => "Issuer Offer documents/Issue Summary Document",
            NseFeedKind::RelatedPartyTransactions => "Related Party Transactions",
            NseFeedKind::Regulation29 => "Regulation 29",
            NseFeedKind::Regulation31 => "Regulation 31",
            NseFeedKind::ReasonForEncumbrance => "Reason For Encumbrance",
            NseFeedKind::SecretarialCompliance => "Secretarial Compliance",
            NseFeedKind::ShareTransfers => "Share Transfers",
            NseFeedKind::ShareholdingPattern => "Shareholding Pattern",
            NseFeedKind::StatementOfDeviation => "Statement of Deviation & Variation",
            NseFeedKind::UnitholdingPatterns => "Unitholding Patterns",
            NseFeedKind::VotingResults => "Voting Results",
            NseFeedKind::Circulars => "Circulars",
        }
    }

    pub fn filename(self) -> &'static str {
        match self {
            NseFeedKind::Announcements => "Online_announcements.xml",
            NseFeedKind::AnnualReports => "Annual_Reports.xml",
            NseFeedKind::BoardMeetings => "Board_Meetings.xml",
            NseFeedKind::Brsr => "brsr.xml",
            NseFeedKind::CorporateActions => "Corporate_action.xml",
            NseFeedKind::CorporateGovernance => "Corporate_Governance.xml",
            NseFeedKind::DailyBuyback => "Daily_Buyback.xml",
            NseFeedKind::FinancialResults => "Financial_Results.xml",
            NseFeedKind::IntegratedFilingFinancials => "Integrated_Filing_Financials.xml",
            NseFeedKind::InsiderTrading => "InsiderTrading.xml",
            NseFeedKind::InvestorComplaints => "Investor_Complaints.xml",
            NseFeedKind::OfferDocuments => "Offer_Documents.xml",
            NseFeedKind::RelatedPartyTransactions => "Related_Party_Trans.xml",
            NseFeedKind::Regulation29 => "Sast_Regulation29.xml",
            NseFeedKind::Regulation31 => "Sast_Regulation31.xml",
            NseFeedKind::ReasonForEncumbrance => "Sast_ReasonForEncumbrance.xml",
            NseFeedKind::SecretarialCompliance => "Secretarial_Compliance.xml",
            NseFeedKind::ShareTransfers => "Share_Transfers.xml",
            NseFeedKind::ShareholdingPattern => "Shareholding_Pattern.xml",
            NseFeedKind::StatementOfDeviation => "Statement_Of_Deviation.xml",
            NseFeedKind::UnitholdingPatterns => "Unitholding_Patterns.xml",
            NseFeedKind::VotingResults => "Voting_Results.xml",
            NseFeedKind::Circulars => "Circulars.xml",
        }
    }

    pub fn url(self) -> String {
        format!("{}/{}", Self::BASE, self.filename())
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.slug() == slug)
    }

    /// Description encodings this feed stores as typed columns (list + sort).
    pub fn extra_list_fields(self) -> &'static [DescriptionField] {
        use DescriptionField as F;
        match self {
            NseFeedKind::Announcements | NseFeedKind::DailyBuyback => &[F::Subject],
            NseFeedKind::AnnualReports | NseFeedKind::UnitholdingPatterns => &[F::AsOnDate],
            NseFeedKind::Brsr => &[F::OriginalSubmissionDate],
            NseFeedKind::CorporateActions => &[
                F::Series,
                F::Purpose,
                F::FaceValue,
                F::RecordDate,
                F::BookClosureStartDate,
                F::BookClosureEndDate,
            ],
            NseFeedKind::FinancialResults => &[
                F::RelatingTo,
                F::AuditedUnaudited,
                F::Cumulative,
                F::Consolidated,
                F::IndAs,
                F::Period,
                F::PeriodEnded,
            ],
            NseFeedKind::IntegratedFilingFinancials => &[F::SubmissionType, F::Remarks],
            NseFeedKind::InvestorComplaints => &[F::ForQuarterEnding],
            NseFeedKind::ReasonForEncumbrance => &[F::EncumberedPromoterNames],
            NseFeedKind::Regulation29 => &[F::AcquirerNames],
            NseFeedKind::Regulation31 => &[F::PromoterNames],
            NseFeedKind::RelatedPartyTransactions | NseFeedKind::StatementOfDeviation => {
                &[F::PeriodEndDate]
            }
            NseFeedKind::SecretarialCompliance => &[F::FinancialYear, F::SubmissionType],
            NseFeedKind::ShareTransfers => &[F::PeriodEnded],
            NseFeedKind::VotingResults => &[F::MeetingDate],
            NseFeedKind::Circulars
            | NseFeedKind::OfferDocuments
            | NseFeedKind::BoardMeetings
            | NseFeedKind::CorporateGovernance
            | NseFeedKind::InsiderTrading
            | NseFeedKind::ShareholdingPattern => &[],
        }
    }

    pub fn shows_description_column(self) -> bool {
        matches!(
            self,
            NseFeedKind::Announcements
                | NseFeedKind::OfferDocuments
                | NseFeedKind::BoardMeetings
                | NseFeedKind::CorporateGovernance
        )
    }
}

/// Per-item filing URL, if the RSS `<link>` is not a channel listing page.
///
/// NSE copies `https://www.nseindia.com/companies-listing/...` onto every
/// Corporate Actions item; other feeds leave `<link>` empty when there is no
/// archive file.
pub fn item_document_link(url: &str) -> Option<&str> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let listing = path.contains("://www.nseindia.com/companies-listing/")
        || path.contains("://www.nseindia.com/resources/");
    (!listing).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::NseFeedKind;
    use std::collections::HashSet;

    #[test]
    fn catalog_has_unique_slugs_and_filenames() {
        let slugs: HashSet<_> = NseFeedKind::ALL.iter().map(|k| k.slug()).collect();
        let files: HashSet<_> = NseFeedKind::ALL.iter().map(|k| k.filename()).collect();
        assert_eq!(slugs.len(), NseFeedKind::ALL.len());
        assert_eq!(files.len(), NseFeedKind::ALL.len());
        assert_eq!(NseFeedKind::ALL.len(), 23);
    }

    #[test]
    fn from_slug_round_trips() {
        for kind in NseFeedKind::ALL {
            assert_eq!(NseFeedKind::from_slug(kind.slug()), Some(*kind));
        }
        assert_eq!(NseFeedKind::from_slug("nope"), None);
    }

    #[test]
    fn item_document_link_skips_listing_pages_and_blanks() {
        use super::item_document_link;
        assert_eq!(item_document_link(""), None);
        assert_eq!(item_document_link("   "), None);
        assert_eq!(
            item_document_link(
                "https://www.nseindia.com/companies-listing/corporate-filings-actions"
            ),
            None
        );
        assert_eq!(
            item_document_link(
                "https://nsearchives.nseindia.com/corporate/xbrl/BRSR_1003_WebXMLFile.xml"
            ),
            Some("https://nsearchives.nseindia.com/corporate/xbrl/BRSR_1003_WebXMLFile.xml")
        );
    }
}
