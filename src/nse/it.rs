//! Insider-trading XBRL instance linked from the Insider Trading RSS feed.

use chrono::NaiveDate;

use super::entities::insider_trading::{ActiveModel as ItAM, Column as ItColumn, Model as ItModel};
use super::xbrl::{
    first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take_clean, take_date, take_pct,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub isin: Option<String>,
    pub company_name: Option<String>,
    pub regulation: Option<String>,
    pub instrument: Option<String>,
    pub person: Option<String>,
    pub category: Option<String>,
    pub txn_type: Option<String>,
    pub qty: Option<String>,
    pub value: Option<String>,
    pub mode: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub prior_qty: Option<String>,
    pub prior_pct: Option<String>,
    pub post_qty: Option<String>,
    pub post_pct: Option<String>,
    pub signatory: Option<String>,
    pub designation: Option<String>,
    pub filing_date: Option<NaiveDate>,
    pub exchange: Option<String>,
}

impl ItFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some() || self.company_name.is_some() || self.person.is_some()
    }

    pub fn apply(&self, am: &mut ItAM) {
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.isin, &self.isin);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt(&mut am.regulation, &self.regulation);
        set_opt(&mut am.instrument, &self.instrument);
        set_opt(&mut am.person, &self.person);
        set_opt(&mut am.category, &self.category);
        set_opt(&mut am.txn_type, &self.txn_type);
        set_opt(&mut am.qty, &self.qty);
        set_opt(&mut am.value, &self.value);
        set_opt(&mut am.mode, &self.mode);
        set_opt_date(&mut am.from_date, self.from_date);
        set_opt_date(&mut am.to_date, self.to_date);
        set_opt(&mut am.prior_qty, &self.prior_qty);
        set_opt(&mut am.prior_pct, &self.prior_pct);
        set_opt(&mut am.post_qty, &self.post_qty);
        set_opt(&mut am.post_pct, &self.post_pct);
        set_opt(&mut am.signatory, &self.signatory);
        set_opt(&mut am.designation, &self.designation);
        set_opt_date(&mut am.filing_date, self.filing_date);
        set_opt(&mut am.exchange, &self.exchange);
    }
}

pub fn parse_it_xbrl(xml: &str) -> ItFacts {
    let map = first_text_facts(xml);
    ItFacts {
        nse_symbol: take_clean(&map, "Symbol").or_else(|| take_clean(&map, "NSESymbol")),
        scrip_code: take_clean(&map, "ScripCode"),
        msei_symbol: take_clean(&map, "MSEISymbol"),
        isin: take_clean(&map, "ISINCode").or_else(|| take_clean(&map, "ISIN")),
        company_name: take_clean(&map, "NameOfTheCompany"),
        regulation: take_clean(&map, "DisclosureUnderRegulation"),
        instrument: take_clean(&map, "TypeOfInstrument"),
        person: take_clean(&map, "NameOfThePerson"),
        category: take_clean(&map, "CategoryOfPerson"),
        txn_type: take_clean(&map, "SecuritiesAcquiredOrDisposedTransactionType"),
        qty: take_clean(&map, "SecuritiesAcquiredOrDisposedNumberOfSecurity"),
        value: take_clean(&map, "SecuritiesAcquiredOrDisposedValueOfSecurity"),
        mode: take_clean(&map, "ModeOfAcquisitionOrDisposal"),
        from_date: take_date(
            &map,
            "DateOfAllotmentAdviceOrAcquisitionOfSharesOrSaleOfSharesSpecifyFromDate",
        ),
        to_date: take_date(
            &map,
            "DateOfAllotmentAdviceOrAcquisitionOfSharesOrSaleOfSharesSpecifyToDate",
        ),
        prior_qty: take_clean(
            &map,
            "SecuritiesHeldPriorToAcquisitionOrDisposalNumberOfSecurity",
        ),
        prior_pct: take_pct(
            &map,
            "SecuritiesHeldPriorToAcquisitionOrDisposalPercentageOfShareholding",
        ),
        post_qty: take_clean(
            &map,
            "SecuritiesHeldPostAcquistionOrDisposalNumberOfSecurity",
        ),
        post_pct: take_pct(
            &map,
            "SecuritiesHeldPostAcquistionOrDisposalPercentageOfShareholding",
        ),
        signatory: take_clean(&map, "NameOfTheSignatory"),
        designation: take_clean(&map, "DesignationOfSignatory"),
        filing_date: take_date(&map, "DateOfFiling"),
        exchange: take_clean(&map, "ExchangeOnWhichTheTradeWasExecuted"),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ItField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    Isin,
    CompanyName,
    Regulation,
    Instrument,
    Person,
    Category,
    TxnType,
    Qty,
    Value,
    Mode,
    FromDate,
    ToDate,
    PriorQty,
    PriorPct,
    PostQty,
    PostPct,
    Signatory,
    Designation,
    FilingDate,
    Exchange,
}

impl ItField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::Person,
        Self::TxnType,
        Self::Qty,
        Self::Mode,
    ];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::Isin,
        Self::Regulation,
        Self::Instrument,
        Self::Person,
        Self::Category,
        Self::TxnType,
        Self::Qty,
        Self::Value,
        Self::Mode,
        Self::FromDate,
        Self::ToDate,
        Self::PriorQty,
        Self::PriorPct,
        Self::PostQty,
        Self::PostPct,
        Self::Exchange,
        Self::Signatory,
        Self::Designation,
        Self::FilingDate,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "ItNseSymbol",
            Self::ScripCode => "ItScripCode",
            Self::MseiSymbol => "ItMseiSymbol",
            Self::Isin => "ItIsin",
            Self::CompanyName => "ItCompanyName",
            Self::Regulation => "ItRegulation",
            Self::Instrument => "ItInstrument",
            Self::Person => "ItPerson",
            Self::Category => "ItCategory",
            Self::TxnType => "ItTxnType",
            Self::Qty => "ItQty",
            Self::Value => "ItValue",
            Self::Mode => "ItMode",
            Self::FromDate => "ItFromDate",
            Self::ToDate => "ItToDate",
            Self::PriorQty => "ItPriorQty",
            Self::PriorPct => "ItPriorPct",
            Self::PostQty => "ItPostQty",
            Self::PostPct => "ItPostPct",
            Self::Signatory => "ItSignatory",
            Self::Designation => "ItDesignation",
            Self::FilingDate => "ItFilingDate",
            Self::Exchange => "ItExchange",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::Isin => "ISIN",
            Self::CompanyName => "Company",
            Self::Regulation => "Regulation",
            Self::Instrument => "Instrument",
            Self::Person => "Person",
            Self::Category => "Category",
            Self::TxnType => "Transaction",
            Self::Qty => "Quantity",
            Self::Value => "Value",
            Self::Mode => "Mode",
            Self::FromDate => "From date",
            Self::ToDate => "To date",
            Self::PriorQty => "Prior quantity",
            Self::PriorPct => "Prior %",
            Self::PostQty => "Post quantity",
            Self::PostPct => "Post %",
            Self::Signatory => "Signatory",
            Self::Designation => "Designation",
            Self::FilingDate => "Filing date",
            Self::Exchange => "Exchange",
        }
    }

    pub fn column(self) -> ItColumn {
        match self {
            Self::NseSymbol => ItColumn::NseSymbol,
            Self::ScripCode => ItColumn::ScripCode,
            Self::MseiSymbol => ItColumn::MseiSymbol,
            Self::Isin => ItColumn::Isin,
            Self::CompanyName => ItColumn::CompanyName,
            Self::Regulation => ItColumn::Regulation,
            Self::Instrument => ItColumn::Instrument,
            Self::Person => ItColumn::Person,
            Self::Category => ItColumn::Category,
            Self::TxnType => ItColumn::TxnType,
            Self::Qty => ItColumn::Qty,
            Self::Value => ItColumn::Value,
            Self::Mode => ItColumn::Mode,
            Self::FromDate => ItColumn::FromDate,
            Self::ToDate => ItColumn::ToDate,
            Self::PriorQty => ItColumn::PriorQty,
            Self::PriorPct => ItColumn::PriorPct,
            Self::PostQty => ItColumn::PostQty,
            Self::PostPct => ItColumn::PostPct,
            Self::Signatory => ItColumn::Signatory,
            Self::Designation => ItColumn::Designation,
            Self::FilingDate => ItColumn::FilingDate,
            Self::Exchange => ItColumn::Exchange,
        }
    }

    pub fn display(self, item: &ItModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::Isin => opt_str(&item.isin),
            Self::CompanyName => opt_str(&item.company_name),
            Self::Regulation => opt_str(&item.regulation),
            Self::Instrument => opt_str(&item.instrument),
            Self::Person => opt_str(&item.person),
            Self::Category => opt_str(&item.category),
            Self::TxnType => opt_str(&item.txn_type),
            Self::Qty => opt_str(&item.qty),
            Self::Value => opt_str(&item.value),
            Self::Mode => opt_str(&item.mode),
            Self::FromDate => opt_date(item.from_date),
            Self::ToDate => opt_date(item.to_date),
            Self::PriorQty => opt_str(&item.prior_qty),
            Self::PriorPct => opt_str(&item.prior_pct),
            Self::PostQty => opt_str(&item.post_qty),
            Self::PostPct => opt_str(&item.post_pct),
            Self::Signatory => opt_str(&item.signatory),
            Self::Designation => opt_str(&item.designation),
            Self::FilingDate => opt_date(item.filing_date),
            Self::Exchange => opt_str(&item.exchange),
        }
    }

    pub fn filter_kind(self) -> crate::list_filters::FilterKind {
        match self {
            Self::FromDate | Self::ToDate | Self::FilingDate => {
                crate::list_filters::FilterKind::Date
            }
            _ => crate::list_filters::FilterKind::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_it_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-bse-fin="http://www.bseindia.com/xbrl/fin/2020-03-31/in-bse-fin" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-bse-fin:ScripCode contextRef="OneI">532290</in-bse-fin:ScripCode>
<in-bse-fin:Symbol contextRef="OneI">BLBLIMITED</in-bse-fin:Symbol>
<in-bse-fin:MSEISymbol contextRef="OneI">NOTLISTED</in-bse-fin:MSEISymbol>
<in-bse-fin:NameOfTheCompany contextRef="OneI">BLB LIMITED</in-bse-fin:NameOfTheCompany>
<in-bse-fin:ISINCode contextRef="OneI">INE791A01024</in-bse-fin:ISINCode>
<in-bse-fin:DisclosureUnderRegulation contextRef="OneI">Regulation 7 (2)</in-bse-fin:DisclosureUnderRegulation>
<in-bse-fin:TypeOfInstrument contextRef="OneI">Equity</in-bse-fin:TypeOfInstrument>
<in-bse-fin:CategoryOfPerson contextRef="OneI">Promoter and Director</in-bse-fin:CategoryOfPerson>
<in-bse-fin:NameOfThePerson contextRef="OneI">Brij Rattan Bagri</in-bse-fin:NameOfThePerson>
<in-bse-fin:SecuritiesHeldPriorToAcquisitionOrDisposalNumberOfSecurity contextRef="OneI">24623562</in-bse-fin:SecuritiesHeldPriorToAcquisitionOrDisposalNumberOfSecurity>
<in-bse-fin:SecuritiesHeldPriorToAcquisitionOrDisposalPercentageOfShareholding contextRef="OneI">0.4658</in-bse-fin:SecuritiesHeldPriorToAcquisitionOrDisposalPercentageOfShareholding>
<in-bse-fin:SecuritiesAcquiredOrDisposedNumberOfSecurity contextRef="OneI">13921</in-bse-fin:SecuritiesAcquiredOrDisposedNumberOfSecurity>
<in-bse-fin:SecuritiesAcquiredOrDisposedValueOfSecurity contextRef="OneI">234290</in-bse-fin:SecuritiesAcquiredOrDisposedValueOfSecurity>
<in-bse-fin:SecuritiesAcquiredOrDisposedTransactionType contextRef="OneI">Buy</in-bse-fin:SecuritiesAcquiredOrDisposedTransactionType>
<in-bse-fin:SecuritiesHeldPostAcquistionOrDisposalNumberOfSecurity contextRef="OneI">24637483</in-bse-fin:SecuritiesHeldPostAcquistionOrDisposalNumberOfSecurity>
<in-bse-fin:SecuritiesHeldPostAcquistionOrDisposalPercentageOfShareholding contextRef="OneI">0.466</in-bse-fin:SecuritiesHeldPostAcquistionOrDisposalPercentageOfShareholding>
<in-bse-fin:DateOfAllotmentAdviceOrAcquisitionOfSharesOrSaleOfSharesSpecifyFromDate contextRef="OneI">2026-09-07</in-bse-fin:DateOfAllotmentAdviceOrAcquisitionOfSharesOrSaleOfSharesSpecifyFromDate>
<in-bse-fin:ModeOfAcquisitionOrDisposal contextRef="OneI">Market Purchase</in-bse-fin:ModeOfAcquisitionOrDisposal>
<in-bse-fin:ExchangeOnWhichTheTradeWasExecuted contextRef="OneI">NSE</in-bse-fin:ExchangeOnWhichTheTradeWasExecuted>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_trade_and_converts_holding_percent() {
        let facts = parse_it_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("BLBLIMITED"));
        assert_eq!(facts.person.as_deref(), Some("Brij Rattan Bagri"));
        assert_eq!(facts.txn_type.as_deref(), Some("Buy"));
        assert_eq!(facts.qty.as_deref(), Some("13921"));
        assert_eq!(facts.mode.as_deref(), Some("Market Purchase"));
        assert_eq!(facts.from_date, NaiveDate::from_ymd_opt(2026, 9, 7));
        assert_eq!(facts.prior_pct.as_deref(), Some("46.58"));
        assert_eq!(facts.post_pct.as_deref(), Some("46.6"));
    }
}
