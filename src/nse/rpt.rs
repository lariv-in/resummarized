//! Related-party-transaction XBRL instance linked from the Related Party Transactions RSS feed.

use chrono::NaiveDate;

use super::entities::related_party_transactions::{
    ActiveModel as RptAM, Column as RptColumn, Model as RptModel,
};
use super::xbrl::{
    all_text_facts, first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take_clean,
    take_date, take_yes_no,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RptFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub company_name: Option<String>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    pub reporting_period: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub has_related_party: Option<String>,
    pub entered_transactions: Option<String>,
    pub transaction_count: Option<String>,
    pub counterparty: Option<String>,
    pub transaction_type: Option<String>,
    pub amount: Option<String>,
}

impl RptFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some() || self.company_name.is_some()
    }

    pub fn apply(&self, am: &mut RptAM) {
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt_date(&mut am.fy_start, self.fy_start);
        set_opt_date(&mut am.fy_end, self.fy_end);
        set_opt(&mut am.reporting_period, &self.reporting_period);
        set_opt_date(&mut am.period_start, self.period_start);
        set_opt_date(&mut am.period_end, self.period_end);
        set_opt(&mut am.has_related_party, &self.has_related_party);
        set_opt(&mut am.entered_transactions, &self.entered_transactions);
        set_opt(&mut am.transaction_count, &self.transaction_count);
        set_opt(&mut am.counterparty, &self.counterparty);
        set_opt(&mut am.transaction_type, &self.transaction_type);
        set_opt(&mut am.amount, &self.amount);
    }
}

pub fn parse_rpt_xbrl(xml: &str) -> RptFacts {
    let map = first_text_facts(xml);
    let txn_n = all_text_facts(xml)
        .iter()
        .filter(|f| f.name == "NameOfCounterParty")
        .count();
    RptFacts {
        nse_symbol: take_clean(&map, "NSESymbol").or_else(|| take_clean(&map, "Symbol")),
        scrip_code: take_clean(&map, "ScripCode"),
        msei_symbol: take_clean(&map, "MSEISymbol"),
        company_name: take_clean(&map, "NameOfTheCompany"),
        fy_start: take_date(&map, "DateOfStartOfFinancialYear"),
        fy_end: take_date(&map, "DateOfEndOfFinancialYear"),
        reporting_period: take_clean(&map, "ReportingPeriod"),
        period_start: take_date(&map, "DateOfStartOfReportingPeriod"),
        period_end: take_date(&map, "DateOfEndOfReportingPeriod"),
        has_related_party: take_yes_no(&map, "WhetherTheCompanyHasAnyRelatedParty"),
        entered_transactions: take_yes_no(
            &map,
            "WhetherTheCompanyHasEnteredIntoAnyRelatedPartyTransactionDuringThePeriod",
        ),
        transaction_count: (txn_n > 0).then(|| txn_n.to_string()),
        counterparty: take_clean(&map, "NameOfCounterParty"),
        transaction_type: take_clean(&map, "TypeOfRelatedPartyTransaction"),
        amount: take_clean(
            &map,
            "AmountOfRelatedPartyTransactionDuringTheReportingPeriod",
        ),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RptField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    CompanyName,
    FyStart,
    FyEnd,
    ReportingPeriod,
    PeriodStart,
    PeriodEnd,
    HasRelatedParty,
    EnteredTransactions,
    TransactionCount,
    Counterparty,
    TransactionType,
    Amount,
}

impl RptField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::ReportingPeriod,
        Self::PeriodEnd,
        Self::TransactionCount,
    ];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::FyStart,
        Self::FyEnd,
        Self::ReportingPeriod,
        Self::PeriodStart,
        Self::PeriodEnd,
        Self::HasRelatedParty,
        Self::EnteredTransactions,
        Self::TransactionCount,
        Self::Counterparty,
        Self::TransactionType,
        Self::Amount,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "RptNseSymbol",
            Self::ScripCode => "RptScripCode",
            Self::MseiSymbol => "RptMseiSymbol",
            Self::CompanyName => "RptCompanyName",
            Self::FyStart => "RptFyStart",
            Self::FyEnd => "RptFyEnd",
            Self::ReportingPeriod => "RptReportingPeriod",
            Self::PeriodStart => "RptPeriodStart",
            Self::PeriodEnd => "RptPeriodEnd",
            Self::HasRelatedParty => "RptHasRelatedParty",
            Self::EnteredTransactions => "RptEnteredTransactions",
            Self::TransactionCount => "RptTransactionCount",
            Self::Counterparty => "RptCounterparty",
            Self::TransactionType => "RptTransactionType",
            Self::Amount => "RptAmount",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::CompanyName => "Company",
            Self::FyStart => "FY start",
            Self::FyEnd => "FY end",
            Self::ReportingPeriod => "Reporting period",
            Self::PeriodStart => "Period start",
            Self::PeriodEnd => "Period end",
            Self::HasRelatedParty => "Has related party",
            Self::EnteredTransactions => "Entered transactions",
            Self::TransactionCount => "Transactions",
            Self::Counterparty => "Counterparty",
            Self::TransactionType => "Transaction type",
            Self::Amount => "Amount",
        }
    }

    pub fn column(self) -> RptColumn {
        match self {
            Self::NseSymbol => RptColumn::NseSymbol,
            Self::ScripCode => RptColumn::ScripCode,
            Self::MseiSymbol => RptColumn::MseiSymbol,
            Self::CompanyName => RptColumn::CompanyName,
            Self::FyStart => RptColumn::FyStart,
            Self::FyEnd => RptColumn::FyEnd,
            Self::ReportingPeriod => RptColumn::ReportingPeriod,
            Self::PeriodStart => RptColumn::PeriodStart,
            Self::PeriodEnd => RptColumn::PeriodEnd,
            Self::HasRelatedParty => RptColumn::HasRelatedParty,
            Self::EnteredTransactions => RptColumn::EnteredTransactions,
            Self::TransactionCount => RptColumn::TransactionCount,
            Self::Counterparty => RptColumn::Counterparty,
            Self::TransactionType => RptColumn::TransactionType,
            Self::Amount => RptColumn::Amount,
        }
    }

    pub fn display(self, item: &RptModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::CompanyName => opt_str(&item.company_name),
            Self::FyStart => opt_date(item.fy_start),
            Self::FyEnd => opt_date(item.fy_end),
            Self::ReportingPeriod => opt_str(&item.reporting_period),
            Self::PeriodStart => opt_date(item.period_start),
            Self::PeriodEnd => opt_date(item.period_end),
            Self::HasRelatedParty => opt_str(&item.has_related_party),
            Self::EnteredTransactions => opt_str(&item.entered_transactions),
            Self::TransactionCount => opt_str(&item.transaction_count),
            Self::Counterparty => opt_str(&item.counterparty),
            Self::TransactionType => opt_str(&item.transaction_type),
            Self::Amount => opt_str(&item.amount),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_rpt_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="http://www.bseindia.com/xbrl/2023-09-30/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-capmkt:NameOfTheCompany contextRef="OneI">Ballarpur Industries Limited</in-capmkt:NameOfTheCompany>
<in-capmkt:ScripCode contextRef="OneI">500102</in-capmkt:ScripCode>
<in-capmkt:NSESymbol contextRef="OneI">BALLARPUR</in-capmkt:NSESymbol>
<in-capmkt:MSEISymbol contextRef="OneI">NOTLISTED</in-capmkt:MSEISymbol>
<in-capmkt:DateOfStartOfFinancialYear contextRef="OneI">2024-04-01</in-capmkt:DateOfStartOfFinancialYear>
<in-capmkt:DateOfEndOfFinancialYear contextRef="OneI">2025-03-31</in-capmkt:DateOfEndOfFinancialYear>
<in-capmkt:ReportingPeriod contextRef="OneI">First half yearly</in-capmkt:ReportingPeriod>
<in-capmkt:DateOfStartOfReportingPeriod contextRef="OneI">2024-04-01</in-capmkt:DateOfStartOfReportingPeriod>
<in-capmkt:DateOfEndOfReportingPeriod contextRef="OneI">2024-09-30</in-capmkt:DateOfEndOfReportingPeriod>
<in-capmkt:WhetherTheCompanyHasAnyRelatedParty contextRef="OneI">true</in-capmkt:WhetherTheCompanyHasAnyRelatedParty>
<in-capmkt:WhetherTheCompanyHasEnteredIntoAnyRelatedPartyTransactionDuringThePeriod contextRef="OneI">true</in-capmkt:WhetherTheCompanyHasEnteredIntoAnyRelatedPartyTransactionDuringThePeriod>
<in-capmkt:NameOfCounterParty contextRef="T1">Hardik Bharat Patel</in-capmkt:NameOfCounterParty>
<in-capmkt:TypeOfRelatedPartyTransaction contextRef="T1">Any other transaction</in-capmkt:TypeOfRelatedPartyTransaction>
<in-capmkt:AmountOfRelatedPartyTransactionDuringTheReportingPeriod contextRef="T1">14730000</in-capmkt:AmountOfRelatedPartyTransactionDuringTheReportingPeriod>
<in-capmkt:NameOfCounterParty contextRef="T2">Other Party</in-capmkt:NameOfCounterParty>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_identity_period_and_first_counterparty() {
        let facts = parse_rpt_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("BALLARPUR"));
        assert_eq!(facts.reporting_period.as_deref(), Some("First half yearly"));
        assert_eq!(facts.period_end, NaiveDate::from_ymd_opt(2024, 9, 30));
        assert_eq!(facts.has_related_party.as_deref(), Some("Yes"));
        assert_eq!(facts.entered_transactions.as_deref(), Some("Yes"));
        assert_eq!(facts.transaction_count.as_deref(), Some("2"));
        assert_eq!(facts.counterparty.as_deref(), Some("Hardik Bharat Patel"));
        assert_eq!(facts.amount.as_deref(), Some("14730000"));
    }
}
