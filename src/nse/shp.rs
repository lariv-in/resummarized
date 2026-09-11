//! Shareholding-pattern XBRL instance linked from the Shareholding Pattern RSS feed.

use chrono::NaiveDate;
use std::collections::HashMap;

use super::entities::shareholding_pattern::{
    ActiveModel as ShpAM, Column as ShpColumn, Model as ShpModel,
};
use super::xbrl::{
    XbrlFact, all_text_facts, opt_date, opt_str, set_opt, set_opt_date, take, take_date,
};

const PROMOTER_CTX: &str = "ShareholdingOfPromoterAndPromoterGroup_ContextI";
const PUBLIC_CTX: &str = "PublicShareholding_ContextI";

/// Document-level identity plus promoter/public summary from Table I.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShpFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub isin: Option<String>,
    pub company_name: Option<String>,
    pub class_of_security: Option<String>,
    pub type_of_report: Option<String>,
    pub date_of_report: Option<NaiveDate>,
    pub filed_under: Option<String>,
    pub promoter_pct: Option<String>,
    pub public_pct: Option<String>,
    pub promoter_shares: Option<String>,
    pub public_shares: Option<String>,
}

impl ShpFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some() || self.isin.is_some() || self.company_name.is_some()
    }

    pub fn apply(&self, am: &mut ShpAM) {
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.isin, &self.isin);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt(&mut am.class_of_security, &self.class_of_security);
        set_opt(&mut am.type_of_report, &self.type_of_report);
        set_opt_date(&mut am.date_of_report, self.date_of_report);
        set_opt(&mut am.filed_under, &self.filed_under);
        set_opt(&mut am.promoter_pct, &self.promoter_pct);
        set_opt(&mut am.public_pct, &self.public_pct);
        set_opt(&mut am.promoter_shares, &self.promoter_shares);
        set_opt(&mut am.public_shares, &self.public_shares);
    }
}

fn take_clean(map: &HashMap<String, String>, key: &str) -> Option<String> {
    take(map, key).filter(|s| !s.bytes().all(|b| b == b'*'))
}

fn named_in_context(facts: &[XbrlFact], name: &str, context: &str) -> Option<String> {
    facts
        .iter()
        .find(|f| f.name == name && f.context == context)
        .map(|f| f.text.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn fraction_to_percent(s: &str) -> String {
    let s = s.trim();
    let Ok(v) = s.parse::<f64>() else {
        return s.to_string();
    };
    let pct = if (0.0..=1.0).contains(&v) {
        v * 100.0
    } else {
        v
    };
    let t = format!("{pct:.4}");
    t.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn pct_in_context(facts: &[XbrlFact], context: &str) -> Option<String> {
    named_in_context(
        facts,
        "ShareholdingAsAPercentageOfTotalNumberOfShares",
        context,
    )
    .map(|s| fraction_to_percent(&s))
}

/// Parse identity and promoter/public summary from a `SHP_*` XBRL instance.
pub fn parse_shp_xbrl(xml: &str) -> ShpFacts {
    let facts = all_text_facts(xml);
    let mut first = HashMap::new();
    for fact in &facts {
        first
            .entry(fact.name.clone())
            .or_insert_with(|| fact.text.clone());
    }
    ShpFacts {
        nse_symbol: take_clean(&first, "Symbol").or_else(|| take_clean(&first, "NSESymbol")),
        scrip_code: take_clean(&first, "ScripCode"),
        msei_symbol: take_clean(&first, "MSEISymbol"),
        isin: take_clean(&first, "ISIN"),
        company_name: take_clean(&first, "NameOfTheCompany"),
        class_of_security: take_clean(&first, "ClassOfSecurity"),
        type_of_report: take_clean(&first, "TypeOfReport"),
        date_of_report: take_date(&first, "DateOfReport"),
        filed_under: take_clean(&first, "ShareholdingPatternFiledUnder"),
        promoter_pct: pct_in_context(&facts, PROMOTER_CTX),
        public_pct: pct_in_context(&facts, PUBLIC_CTX),
        promoter_shares: named_in_context(&facts, "NumberOfFullyPaidUpEquityShares", PROMOTER_CTX),
        public_shares: named_in_context(&facts, "NumberOfFullyPaidUpEquityShares", PUBLIC_CTX),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShpField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    Isin,
    CompanyName,
    ClassOfSecurity,
    TypeOfReport,
    DateOfReport,
    FiledUnder,
    PromoterPct,
    PublicPct,
    PromoterShares,
    PublicShares,
}

impl ShpField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::DateOfReport,
        Self::PromoterPct,
        Self::PublicPct,
    ];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::Isin,
        Self::ClassOfSecurity,
        Self::TypeOfReport,
        Self::DateOfReport,
        Self::FiledUnder,
        Self::PromoterPct,
        Self::PublicPct,
        Self::PromoterShares,
        Self::PublicShares,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "ShpNseSymbol",
            Self::ScripCode => "ShpScripCode",
            Self::MseiSymbol => "ShpMseiSymbol",
            Self::Isin => "ShpIsin",
            Self::CompanyName => "ShpCompanyName",
            Self::ClassOfSecurity => "ShpClassOfSecurity",
            Self::TypeOfReport => "ShpTypeOfReport",
            Self::DateOfReport => "ShpDateOfReport",
            Self::FiledUnder => "ShpFiledUnder",
            Self::PromoterPct => "ShpPromoterPct",
            Self::PublicPct => "ShpPublicPct",
            Self::PromoterShares => "ShpPromoterShares",
            Self::PublicShares => "ShpPublicShares",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::Isin => "ISIN",
            Self::CompanyName => "Company",
            Self::ClassOfSecurity => "Class of security",
            Self::TypeOfReport => "Type of report",
            Self::DateOfReport => "Date of report",
            Self::FiledUnder => "Filed under",
            Self::PromoterPct => "Promoter %",
            Self::PublicPct => "Public %",
            Self::PromoterShares => "Promoter shares",
            Self::PublicShares => "Public shares",
        }
    }

    pub fn column(self) -> ShpColumn {
        match self {
            Self::NseSymbol => ShpColumn::NseSymbol,
            Self::ScripCode => ShpColumn::ScripCode,
            Self::MseiSymbol => ShpColumn::MseiSymbol,
            Self::Isin => ShpColumn::Isin,
            Self::CompanyName => ShpColumn::CompanyName,
            Self::ClassOfSecurity => ShpColumn::ClassOfSecurity,
            Self::TypeOfReport => ShpColumn::TypeOfReport,
            Self::DateOfReport => ShpColumn::DateOfReport,
            Self::FiledUnder => ShpColumn::FiledUnder,
            Self::PromoterPct => ShpColumn::PromoterPct,
            Self::PublicPct => ShpColumn::PublicPct,
            Self::PromoterShares => ShpColumn::PromoterShares,
            Self::PublicShares => ShpColumn::PublicShares,
        }
    }

    pub fn display(self, item: &ShpModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::Isin => opt_str(&item.isin),
            Self::CompanyName => opt_str(&item.company_name),
            Self::ClassOfSecurity => opt_str(&item.class_of_security),
            Self::TypeOfReport => opt_str(&item.type_of_report),
            Self::DateOfReport => opt_date(item.date_of_report),
            Self::FiledUnder => opt_str(&item.filed_under),
            Self::PromoterPct => opt_str(&item.promoter_pct),
            Self::PublicPct => opt_str(&item.public_pct),
            Self::PromoterShares => opt_str(&item.promoter_shares),
            Self::PublicShares => opt_str(&item.public_shares),
        }
    }

    pub fn filter_kind(self) -> crate::list_filters::FilterKind {
        match self {
            Self::DateOfReport => crate::list_filters::FilterKind::Date,
            _ => crate::list_filters::FilterKind::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_shp_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-bse-shp="http://www.bseindia.com/xbrl/shp/2025-10-31/in-bse-shp" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-bse-shp:ScripCode contextRef="MainD">000000</in-bse-shp:ScripCode>
<in-bse-shp:Symbol contextRef="MainD">SRPL</in-bse-shp:Symbol>
<in-bse-shp:MSEISymbol contextRef="MainD">NOTLISTED</in-bse-shp:MSEISymbol>
<in-bse-shp:ISIN contextRef="MainD">INE008Z01012</in-bse-shp:ISIN>
<in-bse-shp:NameOfTheCompany contextRef="MainD">Shree Ram Proteins Limited</in-bse-shp:NameOfTheCompany>
<in-bse-shp:WhetherCompanyIsSME contextRef="MainI">******</in-bse-shp:WhetherCompanyIsSME>
<in-bse-shp:ClassOfSecurity contextRef="MainD">Equity Shares</in-bse-shp:ClassOfSecurity>
<in-bse-shp:TypeOfReport contextRef="MainD">Quarterly</in-bse-shp:TypeOfReport>
<in-bse-shp:DateOfReport contextRef="MainI">2026-06-30</in-bse-shp:DateOfReport>
<in-bse-shp:ShareholdingPatternFiledUnder contextRef="MainD">Regulation 31 (1) (b)</in-bse-shp:ShareholdingPatternFiledUnder>
<in-bse-shp:ShareholdingAsAPercentageOfTotalNumberOfShares contextRef="ShareholdingOfPromoterAndPromoterGroup_ContextI" unitRef="pure" decimals="4">0.0408</in-bse-shp:ShareholdingAsAPercentageOfTotalNumberOfShares>
<in-bse-shp:NumberOfFullyPaidUpEquityShares contextRef="ShareholdingOfPromoterAndPromoterGroup_ContextI" unitRef="shares" decimals="INF">8734679</in-bse-shp:NumberOfFullyPaidUpEquityShares>
<in-bse-shp:ShareholdingAsAPercentageOfTotalNumberOfShares contextRef="PublicShareholding_ContextI" unitRef="pure" decimals="4">0.9592</in-bse-shp:ShareholdingAsAPercentageOfTotalNumberOfShares>
<in-bse-shp:NumberOfFullyPaidUpEquityShares contextRef="PublicShareholding_ContextI" unitRef="shares" decimals="INF">205465321</in-bse-shp:NumberOfFullyPaidUpEquityShares>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_identity_and_promoter_public_summary() {
        let facts = parse_shp_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("SRPL"));
        assert_eq!(facts.scrip_code.as_deref(), Some("000000"));
        assert_eq!(facts.msei_symbol.as_deref(), Some("NOTLISTED"));
        assert_eq!(facts.isin.as_deref(), Some("INE008Z01012"));
        assert_eq!(
            facts.company_name.as_deref(),
            Some("Shree Ram Proteins Limited")
        );
        assert_eq!(facts.class_of_security.as_deref(), Some("Equity Shares"));
        assert_eq!(facts.type_of_report.as_deref(), Some("Quarterly"));
        assert_eq!(facts.date_of_report, NaiveDate::from_ymd_opt(2026, 6, 30));
        assert_eq!(facts.filed_under.as_deref(), Some("Regulation 31 (1) (b)"));
        assert_eq!(facts.promoter_pct.as_deref(), Some("4.08"));
        assert_eq!(facts.public_pct.as_deref(), Some("95.92"));
        assert_eq!(facts.promoter_shares.as_deref(), Some("8734679"));
        assert_eq!(facts.public_shares.as_deref(), Some("205465321"));
    }

    #[test]
    fn skips_all_asterisk_placeholders() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-bse-shp="http://www.bseindia.com/xbrl/shp/2025-10-31/in-bse-shp" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-bse-shp:Symbol contextRef="MainD">SRPL</in-bse-shp:Symbol>
<in-bse-shp:ClassOfSecurity contextRef="MainD">******</in-bse-shp:ClassOfSecurity>
<in-bse-shp:TypeOfReport contextRef="MainD">******</in-bse-shp:TypeOfReport>
<in-bse-shp:ShareholdingPatternFiledUnder contextRef="MainD">******</in-bse-shp:ShareholdingPatternFiledUnder>
</xbrli:xbrl>
"#;
        let facts = parse_shp_xbrl(xml);
        assert_eq!(facts.nse_symbol.as_deref(), Some("SRPL"));
        assert!(facts.class_of_security.is_none());
        assert!(facts.type_of_report.is_none());
        assert!(facts.filed_under.is_none());
    }
}
