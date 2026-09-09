//! Investor-complaints XBRL instance linked from the Investor Complaints RSS feed.

use chrono::NaiveDate;

use super::entities::item::{ActiveModel as ItemAM, Column as ItemColumn, Model as ItemModel};
use super::xbrl::{
    first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take_clean, take_date, take_yes_no,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IcFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub isin: Option<String>,
    pub company_name: Option<String>,
    pub class: Option<String>,
    pub period_end: Option<NaiveDate>,
    pub submission_type: Option<String>,
    pub pending_start: Option<String>,
    pub received: Option<String>,
    pub disposed: Option<String>,
    pub pending_end: Option<String>,
    pub scores_id: Option<String>,
}

impl IcFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some() || self.company_name.is_some() || self.isin.is_some()
    }

    pub fn apply(&self, am: &mut ItemAM) {
        set_opt(&mut am.ic_nse_symbol, &self.nse_symbol);
        set_opt(&mut am.ic_scrip_code, &self.scrip_code);
        set_opt(&mut am.ic_msei_symbol, &self.msei_symbol);
        set_opt(&mut am.ic_isin, &self.isin);
        set_opt(&mut am.ic_company_name, &self.company_name);
        set_opt(&mut am.ic_class, &self.class);
        set_opt_date(&mut am.ic_period_end, self.period_end);
        set_opt(&mut am.ic_submission_type, &self.submission_type);
        set_opt(&mut am.ic_pending_start, &self.pending_start);
        set_opt(&mut am.ic_received, &self.received);
        set_opt(&mut am.ic_disposed, &self.disposed);
        set_opt(&mut am.ic_pending_end, &self.pending_end);
        set_opt(&mut am.ic_scores_id, &self.scores_id);
    }
}

pub fn parse_ic_xbrl(xml: &str) -> IcFacts {
    let map = first_text_facts(xml);
    let scores = if take_yes_no(&map, "IsSCORESIDAvailable").as_deref() == Some("Yes") {
        take_clean(&map, "SCORESRegistrationID")
    } else {
        None
    };
    IcFacts {
        nse_symbol: take_clean(&map, "NSESymbol").or_else(|| take_clean(&map, "Symbol")),
        scrip_code: take_clean(&map, "ScripCode"),
        msei_symbol: take_clean(&map, "MSEISymbol"),
        isin: take_clean(&map, "ISIN"),
        company_name: take_clean(&map, "NameOfTheCompany"),
        class: take_clean(&map, "ClassOfSecurityDebtOrEquity"),
        period_end: take_date(&map, "DateOfEndOfReportingPeriod"),
        submission_type: take_clean(&map, "TypeOfSubmission"),
        pending_start: take_clean(&map, "NoOfInvestorComplaints"),
        received: take_clean(&map, "NoOfInvestorComplaintsReceivedDuringThePeriod"),
        disposed: take_clean(&map, "NoOfInvestorComplaintsDisposedOffDuringThePeriod"),
        pending_end: take_clean(&map, "NoOfInvestorComplaintsDuringThePeriod"),
        scores_id: scores,
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IcField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    Isin,
    CompanyName,
    Class,
    PeriodEnd,
    SubmissionType,
    PendingStart,
    Received,
    Disposed,
    PendingEnd,
    ScoresId,
}

impl IcField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::Class,
        Self::Received,
        Self::PendingEnd,
    ];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::Isin,
        Self::Class,
        Self::PeriodEnd,
        Self::SubmissionType,
        Self::PendingStart,
        Self::Received,
        Self::Disposed,
        Self::PendingEnd,
        Self::ScoresId,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "IcNseSymbol",
            Self::ScripCode => "IcScripCode",
            Self::MseiSymbol => "IcMseiSymbol",
            Self::Isin => "IcIsin",
            Self::CompanyName => "IcCompanyName",
            Self::Class => "IcClass",
            Self::PeriodEnd => "IcPeriodEnd",
            Self::SubmissionType => "IcSubmissionType",
            Self::PendingStart => "IcPendingStart",
            Self::Received => "IcReceived",
            Self::Disposed => "IcDisposed",
            Self::PendingEnd => "IcPendingEnd",
            Self::ScoresId => "IcScoresId",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::Isin => "ISIN",
            Self::CompanyName => "Company",
            Self::Class => "Class",
            Self::PeriodEnd => "Period end",
            Self::SubmissionType => "Submission type",
            Self::PendingStart => "Pending start",
            Self::Received => "Received",
            Self::Disposed => "Disposed",
            Self::PendingEnd => "Pending end",
            Self::ScoresId => "SCORES ID",
        }
    }

    pub fn column(self) -> ItemColumn {
        match self {
            Self::NseSymbol => ItemColumn::IcNseSymbol,
            Self::ScripCode => ItemColumn::IcScripCode,
            Self::MseiSymbol => ItemColumn::IcMseiSymbol,
            Self::Isin => ItemColumn::IcIsin,
            Self::CompanyName => ItemColumn::IcCompanyName,
            Self::Class => ItemColumn::IcClass,
            Self::PeriodEnd => ItemColumn::IcPeriodEnd,
            Self::SubmissionType => ItemColumn::IcSubmissionType,
            Self::PendingStart => ItemColumn::IcPendingStart,
            Self::Received => ItemColumn::IcReceived,
            Self::Disposed => ItemColumn::IcDisposed,
            Self::PendingEnd => ItemColumn::IcPendingEnd,
            Self::ScoresId => ItemColumn::IcScoresId,
        }
    }

    pub fn display(self, item: &ItemModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.ic_nse_symbol),
            Self::ScripCode => opt_str(&item.ic_scrip_code),
            Self::MseiSymbol => opt_str(&item.ic_msei_symbol),
            Self::Isin => opt_str(&item.ic_isin),
            Self::CompanyName => opt_str(&item.ic_company_name),
            Self::Class => opt_str(&item.ic_class),
            Self::PeriodEnd => opt_date(item.ic_period_end),
            Self::SubmissionType => opt_str(&item.ic_submission_type),
            Self::PendingStart => opt_str(&item.ic_pending_start),
            Self::Received => opt_str(&item.ic_received),
            Self::Disposed => opt_str(&item.ic_disposed),
            Self::PendingEnd => opt_str(&item.ic_pending_end),
            Self::ScoresId => opt_str(&item.ic_scores_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_ic_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="http://www.bseindia.com/xbrl/2023-09-30/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-capmkt:ClassOfSecurityDebtOrEquity contextRef="OneI">Debt</in-capmkt:ClassOfSecurityDebtOrEquity>
<in-capmkt:NSESymbol contextRef="OneI">NFIS</in-capmkt:NSESymbol>
<in-capmkt:NameOfTheCompany contextRef="OneI">Nomura Fixed Income Securities Limited</in-capmkt:NameOfTheCompany>
<in-capmkt:ScripCode contextRef="OneI">000000</in-capmkt:ScripCode>
<in-capmkt:MSEISymbol contextRef="OneI">NOTLISTED</in-capmkt:MSEISymbol>
<in-capmkt:ISIN contextRef="OneI">INE127K08017</in-capmkt:ISIN>
<in-capmkt:IsSCORESIDAvailable contextRef="OneI">true</in-capmkt:IsSCORESIDAvailable>
<in-capmkt:SCORESRegistrationID contextRef="OneI">comn00559</in-capmkt:SCORESRegistrationID>
<in-capmkt:DateOfEndOfReportingPeriod contextRef="OneI">2026-06-30</in-capmkt:DateOfEndOfReportingPeriod>
<in-capmkt:TypeOfSubmission contextRef="OneI">Original</in-capmkt:TypeOfSubmission>
<in-capmkt:NoOfInvestorComplaints contextRef="OneI">1</in-capmkt:NoOfInvestorComplaints>
<in-capmkt:NoOfInvestorComplaintsReceivedDuringThePeriod contextRef="OneI">2</in-capmkt:NoOfInvestorComplaintsReceivedDuringThePeriod>
<in-capmkt:NoOfInvestorComplaintsDisposedOffDuringThePeriod contextRef="OneI">2</in-capmkt:NoOfInvestorComplaintsDisposedOffDuringThePeriod>
<in-capmkt:NoOfInvestorComplaintsDuringThePeriod contextRef="OneI">1</in-capmkt:NoOfInvestorComplaintsDuringThePeriod>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_counts_and_scores() {
        let facts = parse_ic_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("NFIS"));
        assert_eq!(facts.class.as_deref(), Some("Debt"));
        assert_eq!(facts.period_end, NaiveDate::from_ymd_opt(2026, 6, 30));
        assert_eq!(facts.received.as_deref(), Some("2"));
        assert_eq!(facts.pending_end.as_deref(), Some("1"));
        assert_eq!(facts.scores_id.as_deref(), Some("comn00559"));
    }

    #[test]
    fn skips_scores_when_unavailable() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="http://www.bseindia.com/xbrl/2023-09-30/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-capmkt:NSESymbol contextRef="OneI">EDUCOMP</in-capmkt:NSESymbol>
<in-capmkt:IsSCORESIDAvailable contextRef="OneI">false</in-capmkt:IsSCORESIDAvailable>
<in-capmkt:SCORESRegistrationID contextRef="OneI">should-skip</in-capmkt:SCORESRegistrationID>
</xbrli:xbrl>
"#;
        let facts = parse_ic_xbrl(xml);
        assert_eq!(facts.nse_symbol.as_deref(), Some("EDUCOMP"));
        assert!(facts.scores_id.is_none());
    }
}
