//! Annual secretarial-compliance XBRL instance linked from the Secretarial Compliance RSS feed.

use chrono::NaiveDate;
use std::collections::HashMap;

use super::entities::secretarial_compliance::{
    ActiveModel as ScrAM, Column as ScrColumn, Model as ScrModel,
};
use super::xbrl::{first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take, take_date};

/// Document-level identity, FY, observation flags, and PCS sign-off.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScrFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub isin: Option<String>,
    pub company_name: Option<String>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    pub date_of_report: Option<NaiveDate>,
    pub observations_reported: Option<String>,
    pub previous_observations: Option<String>,
    pub actions_taken: Option<String>,
    pub certifying_firm: Option<String>,
    pub pcs_name: Option<String>,
    pub membership_type: Option<String>,
    pub membership_number: Option<String>,
    pub udin: Option<String>,
    pub cp_number: Option<String>,
    pub place: Option<String>,
    pub pcs_report_date: Option<NaiveDate>,
}

impl ScrFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some() || self.isin.is_some() || self.company_name.is_some()
    }

    pub fn apply(&self, am: &mut ScrAM) {
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.isin, &self.isin);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt_date(&mut am.fy_start, self.fy_start);
        set_opt_date(&mut am.fy_end, self.fy_end);
        set_opt_date(&mut am.date_of_report, self.date_of_report);
        set_opt(&mut am.observations_reported, &self.observations_reported);
        set_opt(&mut am.previous_observations, &self.previous_observations);
        set_opt(&mut am.actions_taken, &self.actions_taken);
        set_opt(&mut am.certifying_firm, &self.certifying_firm);
        set_opt(&mut am.pcs_name, &self.pcs_name);
        set_opt(&mut am.membership_type, &self.membership_type);
        set_opt(&mut am.membership_number, &self.membership_number);
        set_opt(&mut am.udin, &self.udin);
        set_opt(&mut am.cp_number, &self.cp_number);
        set_opt(&mut am.place, &self.place);
        set_opt_date(&mut am.pcs_report_date, self.pcs_report_date);
    }
}

fn take_clean(map: &HashMap<String, String>, key: &str) -> Option<String> {
    take(map, key).filter(|s| !s.bytes().all(|b| b == b'*'))
}

fn take_yes_no(map: &HashMap<String, String>, key: &str) -> Option<String> {
    take_clean(map, key).map(|s| match s.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" => "Yes".to_string(),
        "false" | "no" | "0" => "No".to_string(),
        _ => s,
    })
}

/// Parse identity, FY, observation flags, and PCS facts from an `SCR_*` XBRL instance.
pub fn parse_scr_xbrl(xml: &str) -> ScrFacts {
    let map = first_text_facts(xml);
    ScrFacts {
        nse_symbol: take_clean(&map, "NSESymbol").or_else(|| take_clean(&map, "Symbol")),
        scrip_code: take_clean(&map, "ScripCode"),
        msei_symbol: take_clean(&map, "MSEISymbol"),
        isin: take_clean(&map, "ISIN"),
        company_name: take_clean(&map, "NameOfTheCompany"),
        fy_start: take_date(&map, "DateOfStartOfFinancialYear"),
        fy_end: take_date(&map, "DateOfEndOfFinancialYear"),
        date_of_report: take_date(&map, "DateOfReport"),
        observations_reported: take_yes_no(
            &map,
            "WhetherAnyObservationsReportedByTheSecretarialAuditor",
        ),
        previous_observations: take_yes_no(&map, "IsthereAnyObservationMadeInThePreviousReport"),
        actions_taken: take_yes_no(
            &map,
            "AnyActionStakenAgainstTheListedEntityItsPromotersDirectorsItsMaterialSubsidiariesEitherBySEBIOrByStockExchangesIncludingUnderTheStandardOperatingProceduresIssuedBySEBIThroughVariousCirculars",
        ),
        certifying_firm: take_clean(&map, "NameOfTheCertifyingFirm"),
        pcs_name: take_clean(&map, "NameOfThePracticingCompanySecretaryIssuingTheReport"),
        membership_type: take_clean(&map, "MembershipType"),
        membership_number: take_clean(&map, "MembershipNumber"),
        udin: take_clean(&map, "UDINOfASCR"),
        cp_number: take_clean(&map, "CertificateOfPracticeNumber"),
        place: take_clean(&map, "PlaceOfPCS"),
        pcs_report_date: take_date(&map, "DateOfReportOfPCS"),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScrField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    Isin,
    CompanyName,
    FyStart,
    FyEnd,
    DateOfReport,
    ObservationsReported,
    PreviousObservations,
    ActionsTaken,
    CertifyingFirm,
    PcsName,
    MembershipType,
    MembershipNumber,
    Udin,
    CpNumber,
    Place,
    PcsReportDate,
}

impl ScrField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::DateOfReport,
        Self::ObservationsReported,
    ];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::Isin,
        Self::FyStart,
        Self::FyEnd,
        Self::DateOfReport,
        Self::ObservationsReported,
        Self::PreviousObservations,
        Self::ActionsTaken,
        Self::CertifyingFirm,
        Self::PcsName,
        Self::MembershipType,
        Self::MembershipNumber,
        Self::Udin,
        Self::CpNumber,
        Self::Place,
        Self::PcsReportDate,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "ScrNseSymbol",
            Self::ScripCode => "ScrScripCode",
            Self::MseiSymbol => "ScrMseiSymbol",
            Self::Isin => "ScrIsin",
            Self::CompanyName => "ScrCompanyName",
            Self::FyStart => "ScrFyStart",
            Self::FyEnd => "ScrFyEnd",
            Self::DateOfReport => "ScrDateOfReport",
            Self::ObservationsReported => "ScrObservationsReported",
            Self::PreviousObservations => "ScrPreviousObservations",
            Self::ActionsTaken => "ScrActionsTaken",
            Self::CertifyingFirm => "ScrCertifyingFirm",
            Self::PcsName => "ScrPcsName",
            Self::MembershipType => "ScrMembershipType",
            Self::MembershipNumber => "ScrMembershipNumber",
            Self::Udin => "ScrUdin",
            Self::CpNumber => "ScrCpNumber",
            Self::Place => "ScrPlace",
            Self::PcsReportDate => "ScrPcsReportDate",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::Isin => "ISIN",
            Self::CompanyName => "Company",
            Self::FyStart => "FY start",
            Self::FyEnd => "FY end",
            Self::DateOfReport => "Date of report",
            Self::ObservationsReported => "Observations",
            Self::PreviousObservations => "Previous observations",
            Self::ActionsTaken => "SEBI/exchange action",
            Self::CertifyingFirm => "Certifying firm",
            Self::PcsName => "PCS",
            Self::MembershipType => "Membership type",
            Self::MembershipNumber => "Membership number",
            Self::Udin => "UDIN",
            Self::CpNumber => "CP number",
            Self::Place => "Place",
            Self::PcsReportDate => "PCS report date",
        }
    }

    pub fn column(self) -> ScrColumn {
        match self {
            Self::NseSymbol => ScrColumn::NseSymbol,
            Self::ScripCode => ScrColumn::ScripCode,
            Self::MseiSymbol => ScrColumn::MseiSymbol,
            Self::Isin => ScrColumn::Isin,
            Self::CompanyName => ScrColumn::CompanyName,
            Self::FyStart => ScrColumn::FyStart,
            Self::FyEnd => ScrColumn::FyEnd,
            Self::DateOfReport => ScrColumn::DateOfReport,
            Self::ObservationsReported => ScrColumn::ObservationsReported,
            Self::PreviousObservations => ScrColumn::PreviousObservations,
            Self::ActionsTaken => ScrColumn::ActionsTaken,
            Self::CertifyingFirm => ScrColumn::CertifyingFirm,
            Self::PcsName => ScrColumn::PcsName,
            Self::MembershipType => ScrColumn::MembershipType,
            Self::MembershipNumber => ScrColumn::MembershipNumber,
            Self::Udin => ScrColumn::Udin,
            Self::CpNumber => ScrColumn::CpNumber,
            Self::Place => ScrColumn::Place,
            Self::PcsReportDate => ScrColumn::PcsReportDate,
        }
    }

    pub fn display(self, item: &ScrModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::Isin => opt_str(&item.isin),
            Self::CompanyName => opt_str(&item.company_name),
            Self::FyStart => opt_date(item.fy_start),
            Self::FyEnd => opt_date(item.fy_end),
            Self::DateOfReport => opt_date(item.date_of_report),
            Self::ObservationsReported => opt_str(&item.observations_reported),
            Self::PreviousObservations => opt_str(&item.previous_observations),
            Self::ActionsTaken => opt_str(&item.actions_taken),
            Self::CertifyingFirm => opt_str(&item.certifying_firm),
            Self::PcsName => opt_str(&item.pcs_name),
            Self::MembershipType => opt_str(&item.membership_type),
            Self::MembershipNumber => opt_str(&item.membership_number),
            Self::Udin => opt_str(&item.udin),
            Self::CpNumber => opt_str(&item.cp_number),
            Self::Place => opt_str(&item.place),
            Self::PcsReportDate => opt_date(item.pcs_report_date),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_scr_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="http://www.bseindia.com/xbrl/2023-09-30/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-capmkt:ScripCode contextRef="ICYMainI">000000</in-capmkt:ScripCode>
<in-capmkt:NSESymbol contextRef="ICYMainI">SUPREMEENG</in-capmkt:NSESymbol>
<in-capmkt:MSEISymbol contextRef="ICYMainI">NOTLISTED</in-capmkt:MSEISymbol>
<in-capmkt:ISIN contextRef="ICYMainI">INE319Z01021</in-capmkt:ISIN>
<in-capmkt:NameOfTheCompany contextRef="ICYMainI">Supreme Engineering Limited</in-capmkt:NameOfTheCompany>
<in-capmkt:DateOfStartOfFinancialYear contextRef="ICYMainI">2025-04-01</in-capmkt:DateOfStartOfFinancialYear>
<in-capmkt:DateOfEndOfFinancialYear contextRef="ICYMainI">2026-03-31</in-capmkt:DateOfEndOfFinancialYear>
<in-capmkt:DateOfReport contextRef="ICYMainI">2026-08-20</in-capmkt:DateOfReport>
<in-capmkt:WhetherAnyObservationsReportedByTheSecretarialAuditor contextRef="ICYMainI">true</in-capmkt:WhetherAnyObservationsReportedByTheSecretarialAuditor>
<in-capmkt:IsthereAnyObservationMadeInThePreviousReport contextRef="ICYMainI">false</in-capmkt:IsthereAnyObservationMadeInThePreviousReport>
<in-capmkt:AnyActionStakenAgainstTheListedEntityItsPromotersDirectorsItsMaterialSubsidiariesEitherBySEBIOrByStockExchangesIncludingUnderTheStandardOperatingProceduresIssuedBySEBIThroughVariousCirculars contextRef="ICYMainI">true</in-capmkt:AnyActionStakenAgainstTheListedEntityItsPromotersDirectorsItsMaterialSubsidiariesEitherBySEBIOrByStockExchangesIncludingUnderTheStandardOperatingProceduresIssuedBySEBIThroughVariousCirculars>
<in-capmkt:NameOfTheCertifyingFirm contextRef="ICYMainI">HRU &amp; Associates</in-capmkt:NameOfTheCertifyingFirm>
<in-capmkt:NameOfThePracticingCompanySecretaryIssuingTheReport contextRef="ICYMainI">Himanshu Upadhyay</in-capmkt:NameOfThePracticingCompanySecretaryIssuingTheReport>
<in-capmkt:MembershipType contextRef="ICYMainI">ACS</in-capmkt:MembershipType>
<in-capmkt:MembershipNumber contextRef="ICYMainI">46800</in-capmkt:MembershipNumber>
<in-capmkt:UDINOfASCR contextRef="ICYMainI">A046800H001166083</in-capmkt:UDINOfASCR>
<in-capmkt:CertificateOfPracticeNumber contextRef="ICYMainI">20259</in-capmkt:CertificateOfPracticeNumber>
<in-capmkt:PlaceOfPCS contextRef="ICYMainI">Mumbai</in-capmkt:PlaceOfPCS>
<in-capmkt:DateOfReportOfPCS contextRef="ICYMainI">2026-08-20</in-capmkt:DateOfReportOfPCS>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_identity_observations_and_pcs() {
        let facts = parse_scr_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("SUPREMEENG"));
        assert_eq!(facts.scrip_code.as_deref(), Some("000000"));
        assert_eq!(facts.msei_symbol.as_deref(), Some("NOTLISTED"));
        assert_eq!(facts.isin.as_deref(), Some("INE319Z01021"));
        assert_eq!(
            facts.company_name.as_deref(),
            Some("Supreme Engineering Limited")
        );
        assert_eq!(facts.fy_start, NaiveDate::from_ymd_opt(2025, 4, 1));
        assert_eq!(facts.fy_end, NaiveDate::from_ymd_opt(2026, 3, 31));
        assert_eq!(facts.date_of_report, NaiveDate::from_ymd_opt(2026, 8, 20));
        assert_eq!(facts.observations_reported.as_deref(), Some("Yes"));
        assert_eq!(facts.previous_observations.as_deref(), Some("No"));
        assert_eq!(facts.actions_taken.as_deref(), Some("Yes"));
        assert_eq!(facts.certifying_firm.as_deref(), Some("HRU & Associates"));
        assert_eq!(facts.pcs_name.as_deref(), Some("Himanshu Upadhyay"));
        assert_eq!(facts.membership_type.as_deref(), Some("ACS"));
        assert_eq!(facts.membership_number.as_deref(), Some("46800"));
        assert_eq!(facts.udin.as_deref(), Some("A046800H001166083"));
        assert_eq!(facts.cp_number.as_deref(), Some("20259"));
        assert_eq!(facts.place.as_deref(), Some("Mumbai"));
        assert_eq!(facts.pcs_report_date, NaiveDate::from_ymd_opt(2026, 8, 20));
    }

    #[test]
    fn skips_all_asterisk_placeholders() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="http://www.bseindia.com/xbrl/2023-09-30/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-capmkt:NSESymbol contextRef="ICYMainI">SUPREMEENG</in-capmkt:NSESymbol>
<in-capmkt:MSEISymbol contextRef="ICYMainI">******</in-capmkt:MSEISymbol>
<in-capmkt:NameOfTheCertifyingFirm contextRef="ICYMainI">******</in-capmkt:NameOfTheCertifyingFirm>
</xbrli:xbrl>
"#;
        let facts = parse_scr_xbrl(xml);
        assert_eq!(facts.nse_symbol.as_deref(), Some("SUPREMEENG"));
        assert!(facts.msei_symbol.is_none());
        assert!(facts.certifying_firm.is_none());
    }
}
