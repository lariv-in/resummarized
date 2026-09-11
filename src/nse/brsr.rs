//! SEBI BRSR XBRL instance linked from the Business Responsibility RSS feed.

use chrono::NaiveDate;
use std::collections::HashMap;

use super::entities::brsr::{ActiveModel as BrsrAM, Column as BrsrColumn, Model as BrsrModel};
use super::xbrl::{first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take, take_date};

/// Section A identity facts from a BRSR `WebXMLFile` XBRL instance.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrsrFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub isin: Option<String>,
    pub cin: Option<String>,
    pub company_name: Option<String>,
    pub date_of_incorporation: Option<NaiveDate>,
    pub registered_office: Option<String>,
    pub corporate_office: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub website: Option<String>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    pub py_start: Option<NaiveDate>,
    pub py_end: Option<NaiveDate>,
    pub ppy_start: Option<NaiveDate>,
    pub ppy_end: Option<NaiveDate>,
    pub paid_up_capital: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub reporting_boundary: Option<String>,
    pub core_assurance: Option<String>,
    pub turnover: Option<String>,
    pub net_worth: Option<String>,
    pub states_served: Option<String>,
    pub countries_served: Option<String>,
    pub board_size: Option<String>,
    pub female_directors: Option<String>,
    pub kmp: Option<String>,
    pub female_kmp: Option<String>,
    pub csr_applicable: Option<String>,
}

impl BrsrFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some()
            || self.cin.is_some()
            || self.isin.is_some()
            || self.company_name.is_some()
    }

    pub fn apply(&self, am: &mut BrsrAM) {
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.isin, &self.isin);
        set_opt(&mut am.cin, &self.cin);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt_date(&mut am.date_of_incorporation, self.date_of_incorporation);
        set_opt(&mut am.registered_office, &self.registered_office);
        set_opt(&mut am.corporate_office, &self.corporate_office);
        set_opt(&mut am.email, &self.email);
        set_opt(&mut am.telephone, &self.telephone);
        set_opt(&mut am.website, &self.website);
        set_opt_date(&mut am.fy_start, self.fy_start);
        set_opt_date(&mut am.fy_end, self.fy_end);
        set_opt_date(&mut am.py_start, self.py_start);
        set_opt_date(&mut am.py_end, self.py_end);
        set_opt_date(&mut am.ppy_start, self.ppy_start);
        set_opt_date(&mut am.ppy_end, self.ppy_end);
        set_opt(&mut am.paid_up_capital, &self.paid_up_capital);
        set_opt(&mut am.contact_person, &self.contact_person);
        set_opt(&mut am.contact_phone, &self.contact_phone);
        set_opt(&mut am.contact_email, &self.contact_email);
        set_opt(&mut am.reporting_boundary, &self.reporting_boundary);
        set_opt(&mut am.core_assurance, &self.core_assurance);
        set_opt(&mut am.turnover, &self.turnover);
        set_opt(&mut am.net_worth, &self.net_worth);
        set_opt(&mut am.states_served, &self.states_served);
        set_opt(&mut am.countries_served, &self.countries_served);
        set_opt(&mut am.board_size, &self.board_size);
        set_opt(&mut am.female_directors, &self.female_directors);
        set_opt(&mut am.kmp, &self.kmp);
        set_opt(&mut am.female_kmp, &self.female_kmp);
        set_opt(&mut am.csr_applicable, &self.csr_applicable);
    }
}

impl From<&HashMap<String, String>> for BrsrFacts {
    fn from(map: &HashMap<String, String>) -> Self {
        Self {
            nse_symbol: take(map, "NSESymbol"),
            scrip_code: take(map, "ScripCode"),
            msei_symbol: take(map, "MSEISymbol"),
            isin: take(map, "ISIN"),
            cin: take(map, "CorporateIdentityNumber"),
            company_name: take(map, "NameOfTheCompany"),
            date_of_incorporation: take_date(map, "DateOfIncorporation"),
            registered_office: take(map, "AddressOfRegisteredOfficeOfCompany"),
            corporate_office: take(map, "AddressOfCorporateOfficeOfCompany"),
            email: take(map, "EMailOfTheCompany"),
            telephone: take(map, "TelephoneOfCompany"),
            website: take(map, "WebsiteOfCompany"),
            fy_start: take_date(map, "DateOfStartOfFinancialYear"),
            fy_end: take_date(map, "DateOfEndOfFinancialYear"),
            py_start: take_date(map, "DateOfStartOfPreviousYear"),
            py_end: take_date(map, "DateOfEndOfPreviousYear"),
            ppy_start: take_date(map, "DateOfStartOfPriorToPreviousYear"),
            ppy_end: take_date(map, "DateOfEndOfPriorToPreviousYear"),
            paid_up_capital: take(map, "ValueOfSharesPaidUp"),
            contact_person: take(map, "NameOfContactPerson"),
            contact_phone: take(map, "ContactNumberOfContactPerson"),
            contact_email: take(map, "EMailOfContactPerson"),
            reporting_boundary: take(map, "ReportingBoundary"),
            core_assurance: take(
                map,
                "WhetherTheCompanyHasUndertakenAssessmentOrAssuranceOfTheBRSRCore",
            ),
            turnover: take(map, "Turnover"),
            net_worth: take(map, "NetWorth"),
            states_served: take(map, "NumberOfStatesWhereMarketServedByTheEntity"),
            countries_served: take(map, "NumberOfCountriesWhereMarketServedByTheEntity"),
            board_size: take(map, "TotalNumberOfBoardOfDirectors"),
            female_directors: take(map, "NumberOfFemaleBoardOfDirectors"),
            kmp: take(map, "TotalNumberOfKeyManagementPersonnel"),
            female_kmp: take(map, "NumberOfFemaleKeyManagementPersonnel"),
            csr_applicable: take(
                map,
                "WhetherCSRIsApplicableAsPerSection135OfCompaniesAct2013",
            ),
        }
    }
}

/// Parse first-seen facts from a BRSR XBRL instance (namespaces ignored).
pub fn parse_brsr_xbrl(xml: &str) -> BrsrFacts {
    BrsrFacts::from(&first_text_facts(xml))
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BrsrField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    Isin,
    Cin,
    CompanyName,
    DateOfIncorporation,
    RegisteredOffice,
    CorporateOffice,
    Email,
    Telephone,
    Website,
    FyStart,
    FyEnd,
    PyStart,
    PyEnd,
    PpyStart,
    PpyEnd,
    PaidUpCapital,
    ContactPerson,
    ContactPhone,
    ContactEmail,
    ReportingBoundary,
    CoreAssurance,
    Turnover,
    NetWorth,
    StatesServed,
    CountriesServed,
    BoardSize,
    FemaleDirectors,
    Kmp,
    FemaleKmp,
    CsrApplicable,
}

impl BrsrField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::Cin,
        Self::Isin,
        Self::FyEnd,
        Self::ReportingBoundary,
    ];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::Isin,
        Self::Cin,
        Self::DateOfIncorporation,
        Self::FyStart,
        Self::FyEnd,
        Self::PyStart,
        Self::PyEnd,
        Self::PpyStart,
        Self::PpyEnd,
        Self::ReportingBoundary,
        Self::PaidUpCapital,
        Self::Turnover,
        Self::NetWorth,
        Self::StatesServed,
        Self::CountriesServed,
        Self::BoardSize,
        Self::FemaleDirectors,
        Self::Kmp,
        Self::FemaleKmp,
        Self::CsrApplicable,
        Self::CoreAssurance,
        Self::RegisteredOffice,
        Self::CorporateOffice,
        Self::Email,
        Self::Telephone,
        Self::Website,
        Self::ContactPerson,
        Self::ContactPhone,
        Self::ContactEmail,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "BrsrNseSymbol",
            Self::ScripCode => "BrsrScripCode",
            Self::MseiSymbol => "BrsrMseiSymbol",
            Self::Isin => "BrsrIsin",
            Self::Cin => "BrsrCin",
            Self::CompanyName => "BrsrCompanyName",
            Self::DateOfIncorporation => "BrsrDateOfIncorporation",
            Self::RegisteredOffice => "BrsrRegisteredOffice",
            Self::CorporateOffice => "BrsrCorporateOffice",
            Self::Email => "BrsrEmail",
            Self::Telephone => "BrsrTelephone",
            Self::Website => "BrsrWebsite",
            Self::FyStart => "BrsrFyStart",
            Self::FyEnd => "BrsrFyEnd",
            Self::PyStart => "BrsrPyStart",
            Self::PyEnd => "BrsrPyEnd",
            Self::PpyStart => "BrsrPpyStart",
            Self::PpyEnd => "BrsrPpyEnd",
            Self::PaidUpCapital => "BrsrPaidUpCapital",
            Self::ContactPerson => "BrsrContactPerson",
            Self::ContactPhone => "BrsrContactPhone",
            Self::ContactEmail => "BrsrContactEmail",
            Self::ReportingBoundary => "BrsrReportingBoundary",
            Self::CoreAssurance => "BrsrCoreAssurance",
            Self::Turnover => "BrsrTurnover",
            Self::NetWorth => "BrsrNetWorth",
            Self::StatesServed => "BrsrStatesServed",
            Self::CountriesServed => "BrsrCountriesServed",
            Self::BoardSize => "BrsrBoardSize",
            Self::FemaleDirectors => "BrsrFemaleDirectors",
            Self::Kmp => "BrsrKmp",
            Self::FemaleKmp => "BrsrFemaleKmp",
            Self::CsrApplicable => "BrsrCsrApplicable",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::Isin => "ISIN",
            Self::Cin => "CIN",
            Self::CompanyName => "Company",
            Self::DateOfIncorporation => "Date of incorporation",
            Self::RegisteredOffice => "Registered office",
            Self::CorporateOffice => "Corporate office",
            Self::Email => "Email",
            Self::Telephone => "Telephone",
            Self::Website => "Website",
            Self::FyStart => "FY start",
            Self::FyEnd => "FY end",
            Self::PyStart => "Previous year start",
            Self::PyEnd => "Previous year end",
            Self::PpyStart => "Prior year start",
            Self::PpyEnd => "Prior year end",
            Self::PaidUpCapital => "Paid-up capital",
            Self::ContactPerson => "Contact person",
            Self::ContactPhone => "Contact phone",
            Self::ContactEmail => "Contact email",
            Self::ReportingBoundary => "Reporting boundary",
            Self::CoreAssurance => "BRSR Core assurance",
            Self::Turnover => "Turnover",
            Self::NetWorth => "Net worth",
            Self::StatesServed => "States served",
            Self::CountriesServed => "Countries served",
            Self::BoardSize => "Board of directors",
            Self::FemaleDirectors => "Female directors",
            Self::Kmp => "KMP",
            Self::FemaleKmp => "Female KMP",
            Self::CsrApplicable => "CSR applicable",
        }
    }

    pub fn column(self) -> BrsrColumn {
        match self {
            Self::NseSymbol => BrsrColumn::NseSymbol,
            Self::ScripCode => BrsrColumn::ScripCode,
            Self::MseiSymbol => BrsrColumn::MseiSymbol,
            Self::Isin => BrsrColumn::Isin,
            Self::Cin => BrsrColumn::Cin,
            Self::CompanyName => BrsrColumn::CompanyName,
            Self::DateOfIncorporation => BrsrColumn::DateOfIncorporation,
            Self::RegisteredOffice => BrsrColumn::RegisteredOffice,
            Self::CorporateOffice => BrsrColumn::CorporateOffice,
            Self::Email => BrsrColumn::Email,
            Self::Telephone => BrsrColumn::Telephone,
            Self::Website => BrsrColumn::Website,
            Self::FyStart => BrsrColumn::FyStart,
            Self::FyEnd => BrsrColumn::FyEnd,
            Self::PyStart => BrsrColumn::PyStart,
            Self::PyEnd => BrsrColumn::PyEnd,
            Self::PpyStart => BrsrColumn::PpyStart,
            Self::PpyEnd => BrsrColumn::PpyEnd,
            Self::PaidUpCapital => BrsrColumn::PaidUpCapital,
            Self::ContactPerson => BrsrColumn::ContactPerson,
            Self::ContactPhone => BrsrColumn::ContactPhone,
            Self::ContactEmail => BrsrColumn::ContactEmail,
            Self::ReportingBoundary => BrsrColumn::ReportingBoundary,
            Self::CoreAssurance => BrsrColumn::CoreAssurance,
            Self::Turnover => BrsrColumn::Turnover,
            Self::NetWorth => BrsrColumn::NetWorth,
            Self::StatesServed => BrsrColumn::StatesServed,
            Self::CountriesServed => BrsrColumn::CountriesServed,
            Self::BoardSize => BrsrColumn::BoardSize,
            Self::FemaleDirectors => BrsrColumn::FemaleDirectors,
            Self::Kmp => BrsrColumn::Kmp,
            Self::FemaleKmp => BrsrColumn::FemaleKmp,
            Self::CsrApplicable => BrsrColumn::CsrApplicable,
        }
    }

    pub fn display(self, item: &BrsrModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::Isin => opt_str(&item.isin),
            Self::Cin => opt_str(&item.cin),
            Self::CompanyName => opt_str(&item.company_name),
            Self::DateOfIncorporation => opt_date(item.date_of_incorporation),
            Self::RegisteredOffice => opt_str(&item.registered_office),
            Self::CorporateOffice => opt_str(&item.corporate_office),
            Self::Email => opt_str(&item.email),
            Self::Telephone => opt_str(&item.telephone),
            Self::Website => opt_str(&item.website),
            Self::FyStart => opt_date(item.fy_start),
            Self::FyEnd => opt_date(item.fy_end),
            Self::PyStart => opt_date(item.py_start),
            Self::PyEnd => opt_date(item.py_end),
            Self::PpyStart => opt_date(item.ppy_start),
            Self::PpyEnd => opt_date(item.ppy_end),
            Self::PaidUpCapital => opt_str(&item.paid_up_capital),
            Self::ContactPerson => opt_str(&item.contact_person),
            Self::ContactPhone => opt_str(&item.contact_phone),
            Self::ContactEmail => opt_str(&item.contact_email),
            Self::ReportingBoundary => opt_str(&item.reporting_boundary),
            Self::CoreAssurance => opt_str(&item.core_assurance),
            Self::Turnover => opt_str(&item.turnover),
            Self::NetWorth => opt_str(&item.net_worth),
            Self::StatesServed => opt_str(&item.states_served),
            Self::CountriesServed => opt_str(&item.countries_served),
            Self::BoardSize => opt_str(&item.board_size),
            Self::FemaleDirectors => opt_str(&item.female_directors),
            Self::Kmp => opt_str(&item.kmp),
            Self::FemaleKmp => opt_str(&item.female_kmp),
            Self::CsrApplicable => opt_str(&item.csr_applicable),
        }
    }

    pub fn filter_kind(self) -> crate::list_filters::FilterKind {
        match self {
            Self::DateOfIncorporation
            | Self::FyStart
            | Self::FyEnd
            | Self::PyStart
            | Self::PyEnd
            | Self::PpyStart
            | Self::PpyEnd => crate::list_filters::FilterKind::Date,
            _ => crate::list_filters::FilterKind::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_brsr_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="https://www.sebi.gov.in/xbrl/2026-02-28/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<xbrli:context id="DCYMain"><xbrli:entity><xbrli:identifier scheme="http://www.nseindia.com">HLEGLAS</xbrli:identifier></xbrli:entity><xbrli:period><xbrli:startDate>2025-04-01</xbrli:startDate><xbrli:endDate>2026-03-31</xbrli:endDate></xbrli:period></xbrli:context>
<in-capmkt:NSESymbol contextRef="ICYMain">HLEGLAS</in-capmkt:NSESymbol>
<in-capmkt:ScripCode contextRef="ICYMain">522215</in-capmkt:ScripCode>
<in-capmkt:MSEISymbol contextRef="ICYMain">NOTLISTED</in-capmkt:MSEISymbol>
<in-capmkt:ISIN contextRef="ICYMain">INE461D01028</in-capmkt:ISIN>
<in-capmkt:CorporateIdentityNumber contextRef="DCYMain">L26100GJ1991PLC016173</in-capmkt:CorporateIdentityNumber>
<in-capmkt:NameOfTheCompany contextRef="ICYMain">HLE Glascoat Limited</in-capmkt:NameOfTheCompany>
<in-capmkt:DateOfIncorporation contextRef="DCYMain">1991-08-26</in-capmkt:DateOfIncorporation>
<in-capmkt:DateOfEndOfFinancialYear contextRef="DCYMain">2026-03-31</in-capmkt:DateOfEndOfFinancialYear>
<in-capmkt:ReportingBoundary contextRef="DCYMain">Standalone basis</in-capmkt:ReportingBoundary>
<in-capmkt:Turnover contextRef="DCYMain" unitRef="INR" decimals="0">7231146410</in-capmkt:Turnover>
<in-capmkt:WhetherCSRIsApplicableAsPerSection135OfCompaniesAct2013 contextRef="DCYMain">true</in-capmkt:WhetherCSRIsApplicableAsPerSection135OfCompaniesAct2013>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_section_a_identity_and_ignores_context_identifier() {
        let facts = parse_brsr_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("HLEGLAS"));
        assert_eq!(facts.scrip_code.as_deref(), Some("522215"));
        assert_eq!(facts.isin.as_deref(), Some("INE461D01028"));
        assert_eq!(facts.cin.as_deref(), Some("L26100GJ1991PLC016173"));
        assert_eq!(facts.company_name.as_deref(), Some("HLE Glascoat Limited"));
        assert_eq!(
            facts.date_of_incorporation,
            NaiveDate::from_ymd_opt(1991, 8, 26)
        );
        assert_eq!(facts.fy_end, NaiveDate::from_ymd_opt(2026, 3, 31));
        assert_eq!(
            facts.reporting_boundary.as_deref(),
            Some("Standalone basis")
        );
        assert_eq!(facts.turnover.as_deref(), Some("7231146410"));
        assert_eq!(facts.csr_applicable.as_deref(), Some("true"));
    }
}
