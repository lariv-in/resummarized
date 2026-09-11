//! Voting-results XBRL instance linked from the Voting Results RSS feed.

use chrono::NaiveDate;
use std::collections::HashMap;

use super::entities::voting_results::{
    ActiveModel as VoteAM, Column as VoteColumn, Model as VoteModel,
};
use super::xbrl::{first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take, take_date};

/// Document-level meeting facts (not per-resolution vote tables).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VoteFacts {
    pub scrip_code: Option<String>,
    pub symbol: Option<String>,
    pub msei_symbol: Option<String>,
    pub isin: Option<String>,
    pub company_name: Option<String>,
    pub type_of_meeting: Option<String>,
    pub date_of_meeting: Option<NaiveDate>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub scrutinizer_name: Option<String>,
    pub scrutinizer_firm: Option<String>,
    pub scrutinizer_qualification: Option<String>,
    pub scrutinizer_membership: Option<String>,
    pub scrutinizer_appointed: Option<NaiveDate>,
    pub report_date: Option<NaiveDate>,
    pub date_of_record: Option<NaiveDate>,
    pub shareholders_on_record: Option<String>,
    pub promoters_in_person: Option<String>,
    pub public_in_person: Option<String>,
    pub promoters_vc: Option<String>,
    pub public_vc: Option<String>,
    pub resolutions_passed: Option<String>,
}

impl VoteFacts {
    pub fn any(&self) -> bool {
        self.symbol.is_some() || self.company_name.is_some() || self.isin.is_some()
    }

    pub fn apply(&self, am: &mut VoteAM) {
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.symbol, &self.symbol);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.isin, &self.isin);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt(&mut am.type_of_meeting, &self.type_of_meeting);
        set_opt_date(&mut am.date_of_meeting, self.date_of_meeting);
        set_opt(&mut am.start_time, &self.start_time);
        set_opt(&mut am.end_time, &self.end_time);
        set_opt(&mut am.scrutinizer, &self.scrutinizer_name);
        set_opt(&mut am.scrutinizer_firm, &self.scrutinizer_firm);
        set_opt(
            &mut am.scrutinizer_qualification,
            &self.scrutinizer_qualification,
        );
        set_opt(&mut am.scrutinizer_membership, &self.scrutinizer_membership);
        set_opt_date(&mut am.board_meeting_date, self.scrutinizer_appointed);
        set_opt_date(&mut am.report_issuance_date, self.report_date);
        set_opt_date(&mut am.record_date, self.date_of_record);
        set_opt(&mut am.shareholders_on_record, &self.shareholders_on_record);
        set_opt(&mut am.promoters_in_person, &self.promoters_in_person);
        set_opt(&mut am.public_in_person, &self.public_in_person);
        set_opt(&mut am.promoters_vc, &self.promoters_vc);
        set_opt(&mut am.public_vc, &self.public_vc);
        set_opt(&mut am.resolutions_passed, &self.resolutions_passed);
    }
}

impl From<&HashMap<String, String>> for VoteFacts {
    fn from(map: &HashMap<String, String>) -> Self {
        Self {
            scrip_code: take(map, "ScripCode"),
            symbol: take(map, "Symbol"),
            msei_symbol: take(map, "MSEISymbol"),
            isin: take(map, "ISIN"),
            company_name: take(map, "NameOfTheCompany"),
            type_of_meeting: take(map, "TypeOfMeeting"),
            date_of_meeting: take_date(map, "DateOfMeeting"),
            start_time: take(map, "StartTimeOfTheMeeting"),
            end_time: take(map, "EndTimeOfTheMeeting"),
            scrutinizer_name: take(map, "NameOfTheScrutinizer"),
            scrutinizer_firm: take(map, "NameOfScrutinizerFirm"),
            scrutinizer_qualification: take(map, "QualificationOfTheScrutinizer"),
            scrutinizer_membership: take(map, "MembershipNumberOfTheScrutinizer"),
            scrutinizer_appointed: take_date(map, "DateOfBoardMeetingInWhichAppointed"),
            report_date: take_date(map, "DateOfIssuanceOfReportToTheCompany"),
            date_of_record: take_date(map, "DateOfRecord"),
            shareholders_on_record: take(map, "TotalNumberOfShareholdersOnRecordDate"),
            promoters_in_person: take(
                map,
                "NumberOfPromotersPresentInTheMeetingEitherInPersonOrThroughProxy",
            ),
            public_in_person: take(
                map,
                "NumberOfPublicShareholdersPresentInTheMeetingEitherInPersonOrThroughProxy",
            ),
            promoters_vc: take(
                map,
                "NumberOfPromotersAttendedTheMeetingThroughVideoConferencing",
            ),
            public_vc: take(
                map,
                "NumberOfPublicShareholdersAttendedTheMeetingThroughVideoConferencing",
            ),
            resolutions_passed: take(map, "NumberOfResolutionPassedInTheMeeting"),
        }
    }
}

pub fn parse_voting_xbrl(xml: &str) -> VoteFacts {
    VoteFacts::from(&first_text_facts(xml))
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VoteField {
    Symbol,
    ScripCode,
    MseiSymbol,
    Isin,
    CompanyName,
    TypeOfMeeting,
    DateOfMeeting,
    StartTime,
    EndTime,
    DateOfRecord,
    ShareholdersOnRecord,
    ResolutionsPassed,
    PromotersInPerson,
    PublicInPerson,
    PromotersVc,
    PublicVc,
    ScrutinizerName,
    ScrutinizerFirm,
    ScrutinizerQualification,
    ScrutinizerMembership,
    ScrutinizerAppointed,
    ReportDate,
}

impl VoteField {
    pub const LIST: &'static [Self] = &[Self::Symbol, Self::TypeOfMeeting, Self::ResolutionsPassed];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::Symbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::Isin,
        Self::TypeOfMeeting,
        Self::DateOfMeeting,
        Self::StartTime,
        Self::EndTime,
        Self::ResolutionsPassed,
        Self::DateOfRecord,
        Self::ShareholdersOnRecord,
        Self::PromotersInPerson,
        Self::PublicInPerson,
        Self::PromotersVc,
        Self::PublicVc,
        Self::ScrutinizerName,
        Self::ScrutinizerFirm,
        Self::ScrutinizerQualification,
        Self::ScrutinizerMembership,
        Self::ScrutinizerAppointed,
        Self::ReportDate,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::Symbol => "VoteSymbol",
            Self::ScripCode => "VoteScripCode",
            Self::MseiSymbol => "VoteMseiSymbol",
            Self::Isin => "VoteIsin",
            Self::CompanyName => "VoteCompanyName",
            Self::TypeOfMeeting => "VoteTypeOfMeeting",
            Self::DateOfMeeting => "VoteDateOfMeeting",
            Self::StartTime => "VoteStartTime",
            Self::EndTime => "VoteEndTime",
            Self::DateOfRecord => "VoteDateOfRecord",
            Self::ShareholdersOnRecord => "VoteShareholdersOnRecord",
            Self::ResolutionsPassed => "VoteResolutionsPassed",
            Self::PromotersInPerson => "VotePromotersInPerson",
            Self::PublicInPerson => "VotePublicInPerson",
            Self::PromotersVc => "VotePromotersVc",
            Self::PublicVc => "VotePublicVc",
            Self::ScrutinizerName => "VoteScrutinizerName",
            Self::ScrutinizerFirm => "VoteScrutinizerFirm",
            Self::ScrutinizerQualification => "VoteScrutinizerQualification",
            Self::ScrutinizerMembership => "VoteScrutinizerMembership",
            Self::ScrutinizerAppointed => "VoteScrutinizerAppointed",
            Self::ReportDate => "VoteReportDate",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Symbol => "Symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::Isin => "ISIN",
            Self::CompanyName => "Company",
            Self::TypeOfMeeting => "Type of meeting",
            Self::DateOfMeeting => "Date of meeting",
            Self::StartTime => "Start time",
            Self::EndTime => "End time",
            Self::DateOfRecord => "Record date",
            Self::ShareholdersOnRecord => "Shareholders on record",
            Self::ResolutionsPassed => "Resolutions passed",
            Self::PromotersInPerson => "Promoters present (in person/proxy)",
            Self::PublicInPerson => "Public present (in person/proxy)",
            Self::PromotersVc => "Promoters attended (VC)",
            Self::PublicVc => "Public attended (VC)",
            Self::ScrutinizerName => "Scrutinizer",
            Self::ScrutinizerFirm => "Scrutinizer firm",
            Self::ScrutinizerQualification => "Scrutinizer qualification",
            Self::ScrutinizerMembership => "Scrutinizer membership",
            Self::ScrutinizerAppointed => "Board meeting (appointment)",
            Self::ReportDate => "Report issuance date",
        }
    }

    pub fn column(self) -> VoteColumn {
        match self {
            Self::Symbol => VoteColumn::Symbol,
            Self::ScripCode => VoteColumn::ScripCode,
            Self::MseiSymbol => VoteColumn::MseiSymbol,
            Self::Isin => VoteColumn::Isin,
            Self::CompanyName => VoteColumn::CompanyName,
            Self::TypeOfMeeting => VoteColumn::TypeOfMeeting,
            Self::DateOfMeeting => VoteColumn::DateOfMeeting,
            Self::StartTime => VoteColumn::StartTime,
            Self::EndTime => VoteColumn::EndTime,
            Self::DateOfRecord => VoteColumn::RecordDate,
            Self::ShareholdersOnRecord => VoteColumn::ShareholdersOnRecord,
            Self::ResolutionsPassed => VoteColumn::ResolutionsPassed,
            Self::PromotersInPerson => VoteColumn::PromotersInPerson,
            Self::PublicInPerson => VoteColumn::PublicInPerson,
            Self::PromotersVc => VoteColumn::PromotersVc,
            Self::PublicVc => VoteColumn::PublicVc,
            Self::ScrutinizerName => VoteColumn::Scrutinizer,
            Self::ScrutinizerFirm => VoteColumn::ScrutinizerFirm,
            Self::ScrutinizerQualification => VoteColumn::ScrutinizerQualification,
            Self::ScrutinizerMembership => VoteColumn::ScrutinizerMembership,
            Self::ScrutinizerAppointed => VoteColumn::BoardMeetingDate,
            Self::ReportDate => VoteColumn::ReportIssuanceDate,
        }
    }

    pub fn display(self, item: &VoteModel) -> String {
        match self {
            Self::Symbol => opt_str(&item.symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::Isin => opt_str(&item.isin),
            Self::CompanyName => opt_str(&item.company_name),
            Self::TypeOfMeeting => opt_str(&item.type_of_meeting),
            Self::DateOfMeeting => opt_date(item.date_of_meeting),
            Self::StartTime => opt_str(&item.start_time),
            Self::EndTime => opt_str(&item.end_time),
            Self::DateOfRecord => opt_date(item.record_date),
            Self::ShareholdersOnRecord => opt_str(&item.shareholders_on_record),
            Self::ResolutionsPassed => opt_str(&item.resolutions_passed),
            Self::PromotersInPerson => opt_str(&item.promoters_in_person),
            Self::PublicInPerson => opt_str(&item.public_in_person),
            Self::PromotersVc => opt_str(&item.promoters_vc),
            Self::PublicVc => opt_str(&item.public_vc),
            Self::ScrutinizerName => opt_str(&item.scrutinizer),
            Self::ScrutinizerFirm => opt_str(&item.scrutinizer_firm),
            Self::ScrutinizerQualification => opt_str(&item.scrutinizer_qualification),
            Self::ScrutinizerMembership => opt_str(&item.scrutinizer_membership),
            Self::ScrutinizerAppointed => opt_date(item.board_meeting_date),
            Self::ReportDate => opt_date(item.report_issuance_date),
        }
    }

    pub fn filter_kind(self) -> crate::list_filters::FilterKind {
        match self {
            Self::DateOfMeeting
            | Self::DateOfRecord
            | Self::ScrutinizerAppointed
            | Self::ReportDate => crate::list_filters::FilterKind::Date,
            _ => crate::list_filters::FilterKind::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_voting_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-bse-voting="http://www.bseindia.com/xbrl/voting/2016-01-12/in-bse-voting" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<xbrli:context id="MainD"><xbrli:entity><xbrli:identifier scheme="http://www.bseindia.com/bse-voting/ScripCode">539854</xbrli:identifier></xbrli:entity><xbrli:period><xbrli:startDate>2026-06-08</xbrli:startDate><xbrli:endDate>2026-09-07</xbrli:endDate></xbrli:period></xbrli:context>
<in-bse-voting:ScripCode contextRef="MainD">539854</in-bse-voting:ScripCode>
<in-bse-voting:Symbol contextRef="MainD">HALDER</in-bse-voting:Symbol>
<in-bse-voting:MSEISymbol contextRef="MainD">NOTLISTED</in-bse-voting:MSEISymbol>
<in-bse-voting:ISIN contextRef="MainD">INE115S01010</in-bse-voting:ISIN>
<in-bse-voting:NameOfTheCompany contextRef="MainD">HALDER VENTURE LIMITED</in-bse-voting:NameOfTheCompany>
<in-bse-voting:TypeOfMeeting contextRef="MainD">AGM</in-bse-voting:TypeOfMeeting>
<in-bse-voting:DateOfMeeting contextRef="MainI">2026-09-07</in-bse-voting:DateOfMeeting>
<in-bse-voting:StartTimeOfTheMeeting contextRef="MainI">11:00 AM</in-bse-voting:StartTimeOfTheMeeting>
<in-bse-voting:EndTimeOfTheMeeting contextRef="MainI">11:52 AM</in-bse-voting:EndTimeOfTheMeeting>
<in-bse-voting:NameOfTheScrutinizer contextRef="MainD">MANOJ PRASAD SHAW</in-bse-voting:NameOfTheScrutinizer>
<in-bse-voting:NameOfScrutinizerFirm contextRef="MainD">MANOJ SHAW &amp; CO.</in-bse-voting:NameOfScrutinizerFirm>
<in-bse-voting:QualificationOfTheScrutinizer contextRef="MainD">CS</in-bse-voting:QualificationOfTheScrutinizer>
<in-bse-voting:MembershipNumberOfTheScrutinizer contextRef="MainD">5517</in-bse-voting:MembershipNumberOfTheScrutinizer>
<in-bse-voting:DateOfBoardMeetingInWhichAppointed contextRef="MainI">2026-05-29</in-bse-voting:DateOfBoardMeetingInWhichAppointed>
<in-bse-voting:DateOfIssuanceOfReportToTheCompany contextRef="MainI">2026-09-07</in-bse-voting:DateOfIssuanceOfReportToTheCompany>
<in-bse-voting:DateOfRecord contextRef="MainI">2026-08-31</in-bse-voting:DateOfRecord>
<in-bse-voting:TotalNumberOfShareholdersOnRecordDate contextRef="MainI" decimals="INF" unitRef="pure">2575</in-bse-voting:TotalNumberOfShareholdersOnRecordDate>
<in-bse-voting:NumberOfPromotersPresentInTheMeetingEitherInPersonOrThroughProxy contextRef="MainI" decimals="INF" unitRef="pure">0</in-bse-voting:NumberOfPromotersPresentInTheMeetingEitherInPersonOrThroughProxy>
<in-bse-voting:NumberOfPublicShareholdersPresentInTheMeetingEitherInPersonOrThroughProxy contextRef="MainI" decimals="INF" unitRef="pure">0</in-bse-voting:NumberOfPublicShareholdersPresentInTheMeetingEitherInPersonOrThroughProxy>
<in-bse-voting:NumberOfPromotersAttendedTheMeetingThroughVideoConferencing contextRef="MainI" decimals="INF" unitRef="pure">3</in-bse-voting:NumberOfPromotersAttendedTheMeetingThroughVideoConferencing>
<in-bse-voting:NumberOfPublicShareholdersAttendedTheMeetingThroughVideoConferencing contextRef="MainI" decimals="INF" unitRef="pure">78</in-bse-voting:NumberOfPublicShareholdersAttendedTheMeetingThroughVideoConferencing>
<in-bse-voting:NumberOfResolutionPassedInTheMeeting contextRef="MainI" decimals="INF" unitRef="pure">6</in-bse-voting:NumberOfResolutionPassedInTheMeeting>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_meeting_identity_and_unescapes_firm() {
        let facts = parse_voting_xbrl(FIXTURE);
        assert_eq!(facts.symbol.as_deref(), Some("HALDER"));
        assert_eq!(facts.scrip_code.as_deref(), Some("539854"));
        assert_eq!(facts.isin.as_deref(), Some("INE115S01010"));
        assert_eq!(
            facts.company_name.as_deref(),
            Some("HALDER VENTURE LIMITED")
        );
        assert_eq!(facts.type_of_meeting.as_deref(), Some("AGM"));
        assert_eq!(facts.date_of_meeting, NaiveDate::from_ymd_opt(2026, 9, 7));
        assert_eq!(facts.start_time.as_deref(), Some("11:00 AM"));
        assert_eq!(facts.end_time.as_deref(), Some("11:52 AM"));
        assert_eq!(facts.scrutinizer_name.as_deref(), Some("MANOJ PRASAD SHAW"));
        assert_eq!(facts.scrutinizer_firm.as_deref(), Some("MANOJ SHAW & CO."));
        assert_eq!(facts.scrutinizer_qualification.as_deref(), Some("CS"));
        assert_eq!(facts.scrutinizer_membership.as_deref(), Some("5517"));
        assert_eq!(
            facts.scrutinizer_appointed,
            NaiveDate::from_ymd_opt(2026, 5, 29)
        );
        assert_eq!(facts.report_date, NaiveDate::from_ymd_opt(2026, 9, 7));
        assert_eq!(facts.date_of_record, NaiveDate::from_ymd_opt(2026, 8, 31));
        assert_eq!(facts.shareholders_on_record.as_deref(), Some("2575"));
        assert_eq!(facts.promoters_in_person.as_deref(), Some("0"));
        assert_eq!(facts.public_in_person.as_deref(), Some("0"));
        assert_eq!(facts.promoters_vc.as_deref(), Some("3"));
        assert_eq!(facts.public_vc.as_deref(), Some("78"));
        assert_eq!(facts.resolutions_passed.as_deref(), Some("6"));
    }
}
