//! SEBI Statement of Deviation & Variation XBRL instance linked from the RSS feed.

use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::entities::statement_of_deviation::{
    ActiveModel as SodAM, Column as SodColumn, Model as SodModel,
};
use super::xbrl::{
    XbrlFact, all_text_facts, opt_date, opt_str, set_opt, set_opt_date, take, take_date,
};

const OBJECT_FIELDS: &[&str] = &[
    "OriginalObject",
    "ModifiedObject",
    "OriginalAllocation",
    "ModifiedAllocation",
    "FundsUtilised",
    "AmountOfDeviationOrVariationForTheQuarterAccordingToApplicableObject",
    "DisclosureNotesOnObjectsForWhichFundsHaveBeenRaisedAndWhereThereHasBeenADeviation",
];

/// One row from the repeating objects-of-issue table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SodObjectRow {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub object: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub modified_object: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub original_allocation: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub modified_allocation: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub funds_utilised: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub amount_of_deviation: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

#[derive(Clone, Copy)]
pub struct ObjectTableColumn {
    pub header: &'static str,
    get: fn(&SodObjectRow) -> &str,
}

impl ObjectTableColumn {
    pub fn cell(self, row: &SodObjectRow) -> &str {
        (self.get)(row)
    }
}

const OBJECT_TABLE_COLUMNS: &[ObjectTableColumn] = &[
    ObjectTableColumn {
        header: "Object",
        get: |r| r.object.as_str(),
    },
    ObjectTableColumn {
        header: "Modified",
        get: |r| r.modified_object.as_str(),
    },
    ObjectTableColumn {
        header: "Original allocation",
        get: |r| r.original_allocation.as_str(),
    },
    ObjectTableColumn {
        header: "Modified allocation",
        get: |r| r.modified_allocation.as_str(),
    },
    ObjectTableColumn {
        header: "Funds utilised",
        get: |r| r.funds_utilised.as_str(),
    },
    ObjectTableColumn {
        header: "Amount of deviation",
        get: |r| r.amount_of_deviation.as_str(),
    },
    ObjectTableColumn {
        header: "Notes",
        get: |r| r.notes.as_str(),
    },
];

/// Columns that have at least one non-empty cell.
pub fn object_table_columns(rows: &[SodObjectRow]) -> Vec<ObjectTableColumn> {
    OBJECT_TABLE_COLUMNS
        .iter()
        .copied()
        .filter(|col| rows.iter().any(|row| !col.cell(row).is_empty()))
        .collect()
}

/// Decode the jsonb objects array stored on an item.
pub fn parse_object_rows(value: &Option<serde_json::Value>) -> Vec<SodObjectRow> {
    value
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// Filing-level facts from a `SOD_*` XBRL instance, plus object-table rows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SodFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub isin: Option<String>,
    pub company_name: Option<String>,
    pub statement_count: Option<String>,
    pub quarter_ended: Option<chrono::NaiveDate>,
    pub mode_of_fund_raising: Option<String>,
    pub date_of_funds_raising: Option<chrono::NaiveDate>,
    pub amount_raised: Option<String>,
    pub monitoring_agency: Option<String>,
    pub monitoring_agency_name: Option<String>,
    pub has_deviation: Option<String>,
    pub deviation_explanation: Option<String>,
    pub shareholder_approved: Option<String>,
    pub audit_committee_comments: Option<String>,
    pub auditor_comments: Option<String>,
    pub objects: Vec<SodObjectRow>,
    pub signatory: Option<String>,
    pub designation: Option<String>,
    pub place: Option<String>,
    pub date_of_signing: Option<chrono::NaiveDate>,
}

impl SodFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some()
            || self.isin.is_some()
            || self.company_name.is_some()
            || self.quarter_ended.is_some()
    }

    pub fn apply(&self, am: &mut SodAM) {
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.isin, &self.isin);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt(&mut am.statement_count, &self.statement_count);
        set_opt_date(&mut am.quarter_ended, self.quarter_ended);
        set_opt(&mut am.mode_of_fund_raising, &self.mode_of_fund_raising);
        set_opt_date(&mut am.date_of_funds_raising, self.date_of_funds_raising);
        set_opt(&mut am.amount_raised, &self.amount_raised);
        set_opt(&mut am.monitoring_agency, &self.monitoring_agency);
        set_opt(&mut am.monitoring_agency_name, &self.monitoring_agency_name);
        set_opt(&mut am.has_deviation, &self.has_deviation);
        set_opt(&mut am.deviation_explanation, &self.deviation_explanation);
        set_opt(&mut am.shareholder_approved, &self.shareholder_approved);
        set_opt(
            &mut am.audit_committee_comments,
            &self.audit_committee_comments,
        );
        set_opt(&mut am.auditor_comments, &self.auditor_comments);
        am.objects = Set(serde_json::to_value(&self.objects).ok());
        set_opt(&mut am.signatory, &self.signatory);
        set_opt(&mut am.designation, &self.designation);
        set_opt(&mut am.place, &self.place);
        set_opt_date(&mut am.date_of_signing, self.date_of_signing);
    }
}

fn join_named(facts: &[XbrlFact], name: &str) -> Option<String> {
    let vals: Vec<String> = facts
        .iter()
        .filter(|f| f.name == name)
        .map(|f| f.text.clone())
        .filter(|s| !s.is_empty())
        .collect();
    if vals.is_empty() {
        None
    } else {
        Some(vals.join(" | "))
    }
}

fn take_deviation(facts: &[XbrlFact]) -> Option<String> {
    let vals: Vec<&str> = facts
        .iter()
        .filter(|f| f.name == "IsThereADeviationOrVariationInUseOfFundsRaised")
        .map(|f| f.text.as_str())
        .collect();
    if vals.is_empty() {
        None
    } else if vals.iter().any(|v| v.eq_ignore_ascii_case("true")) {
        Some("true".to_string())
    } else {
        Some(vals[0].to_string())
    }
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn take_group(map: &HashMap<String, String>, name: &str) -> String {
    map.get(name)
        .map(|s| collapse_ws(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_default()
}

fn object_rows(facts: &[XbrlFact]) -> Vec<SodObjectRow> {
    object_groups(facts)
        .into_iter()
        .map(|map| SodObjectRow {
            object: take_group(&map, "OriginalObject"),
            modified_object: take_group(&map, "ModifiedObject"),
            original_allocation: take_group(&map, "OriginalAllocation"),
            modified_allocation: take_group(&map, "ModifiedAllocation"),
            funds_utilised: take_group(&map, "FundsUtilised"),
            amount_of_deviation: take_group(
                &map,
                "AmountOfDeviationOrVariationForTheQuarterAccordingToApplicableObject",
            ),
            notes: take_group(
                &map,
                "DisclosureNotesOnObjectsForWhichFundsHaveBeenRaisedAndWhereThereHasBeenADeviation",
            ),
        })
        .collect()
}

fn object_groups(facts: &[XbrlFact]) -> Vec<HashMap<String, String>> {
    let mut order = Vec::new();
    let mut groups: HashMap<String, HashMap<String, String>> = HashMap::new();
    for fact in facts {
        if !OBJECT_FIELDS.contains(&fact.name.as_str()) {
            continue;
        }
        if !groups.contains_key(&fact.context) {
            order.push(fact.context.clone());
        }
        groups
            .entry(fact.context.clone())
            .or_default()
            .entry(fact.name.clone())
            .or_insert_with(|| fact.text.clone());
    }
    order
        .into_iter()
        .filter_map(|ctx| {
            let map = groups.remove(&ctx)?;
            let obj = map.get("OriginalObject").map(|s| s.as_str()).unwrap_or("");
            if obj.is_empty()
                && map.get("OriginalAllocation").is_none()
                && map.get("FundsUtilised").is_none()
            {
                None
            } else {
                Some(map)
            }
        })
        .collect()
}

/// Parse identity, first statement, and object-table rows from an SOD XBRL instance.
pub fn parse_sod_xbrl(xml: &str) -> SodFacts {
    let facts = all_text_facts(xml);
    let mut first = HashMap::new();
    for fact in &facts {
        first
            .entry(fact.name.clone())
            .or_insert_with(|| fact.text.clone());
    }
    SodFacts {
        nse_symbol: take(&first, "NSESymbol"),
        scrip_code: take(&first, "ScripCode"),
        msei_symbol: take(&first, "MSEISymbol"),
        isin: take(&first, "ISIN"),
        company_name: take(&first, "NameOfTheCompany"),
        statement_count: take(&first, "NumberOfStatementsOfDeviation"),
        quarter_ended: take_date(&first, "ReportFiledForQuarterEnded"),
        mode_of_fund_raising: join_named(&facts, "ModeOfFundRaising"),
        date_of_funds_raising: take_date(&first, "DateOfFundsRaising"),
        amount_raised: join_named(&facts, "AmountRaised"),
        monitoring_agency: take(&first, "MonitoringAgency"),
        monitoring_agency_name: take(&first, "NameOfMonitoringAgency"),
        has_deviation: take_deviation(&facts),
        deviation_explanation: take(&first, "ExplanationForTheDeviationOrVariation"),
        shareholder_approved: take(
            &first,
            "WhetherTheDeviationOrVariationInUseOfFundsIsPursuantToChangeInTermsOfAContractOrObjectsWhichWasApprovedByTheShareholders",
        ),
        audit_committee_comments: take(&first, "CommentsOfTheAuditCommitteeAfterReview"),
        auditor_comments: take(&first, "CommentsOfTheAuditors"),
        objects: object_rows(&facts),
        signatory: take(&first, "NameOfSignatory"),
        designation: take(&first, "DesignationOfPerson"),
        place: take(&first, "Place"),
        date_of_signing: take_date(&first, "DateOfSigning"),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SodField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    Isin,
    CompanyName,
    StatementCount,
    QuarterEnded,
    ModeOfFundRaising,
    DateOfFundsRaising,
    AmountRaised,
    MonitoringAgency,
    MonitoringAgencyName,
    HasDeviation,
    DeviationExplanation,
    ShareholderApproved,
    AuditCommitteeComments,
    AuditorComments,
    Signatory,
    Designation,
    Place,
    DateOfSigning,
}

impl SodField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::ModeOfFundRaising,
        Self::HasDeviation,
        Self::Isin,
    ];

    pub const DETAIL_HEAD: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::Isin,
        Self::QuarterEnded,
        Self::StatementCount,
        Self::ModeOfFundRaising,
        Self::DateOfFundsRaising,
        Self::AmountRaised,
        Self::MonitoringAgency,
        Self::MonitoringAgencyName,
        Self::HasDeviation,
        Self::ShareholderApproved,
        Self::DeviationExplanation,
        Self::AuditCommitteeComments,
        Self::AuditorComments,
    ];

    pub const DETAIL_TAIL: &'static [Self] = &[
        Self::Signatory,
        Self::Designation,
        Self::Place,
        Self::DateOfSigning,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "SodNseSymbol",
            Self::ScripCode => "SodScripCode",
            Self::MseiSymbol => "SodMseiSymbol",
            Self::Isin => "SodIsin",
            Self::CompanyName => "SodCompanyName",
            Self::StatementCount => "SodStatementCount",
            Self::QuarterEnded => "SodQuarterEnded",
            Self::ModeOfFundRaising => "SodModeOfFundRaising",
            Self::DateOfFundsRaising => "SodDateOfFundsRaising",
            Self::AmountRaised => "SodAmountRaised",
            Self::MonitoringAgency => "SodMonitoringAgency",
            Self::MonitoringAgencyName => "SodMonitoringAgencyName",
            Self::HasDeviation => "SodHasDeviation",
            Self::DeviationExplanation => "SodDeviationExplanation",
            Self::ShareholderApproved => "SodShareholderApproved",
            Self::AuditCommitteeComments => "SodAuditCommitteeComments",
            Self::AuditorComments => "SodAuditorComments",
            Self::Signatory => "SodSignatory",
            Self::Designation => "SodDesignation",
            Self::Place => "SodPlace",
            Self::DateOfSigning => "SodDateOfSigning",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::Isin => "ISIN",
            Self::CompanyName => "Company",
            Self::StatementCount => "Statements",
            Self::QuarterEnded => "Quarter ended",
            Self::ModeOfFundRaising => "Mode of fund raising",
            Self::DateOfFundsRaising => "Date of funds raising",
            Self::AmountRaised => "Amount raised",
            Self::MonitoringAgency => "Monitoring agency",
            Self::MonitoringAgencyName => "Monitoring agency name",
            Self::HasDeviation => "Deviation / variation",
            Self::DeviationExplanation => "Deviation explanation",
            Self::ShareholderApproved => "Shareholder-approved change",
            Self::AuditCommitteeComments => "Audit committee comments",
            Self::AuditorComments => "Auditor comments",
            Self::Signatory => "Signatory",
            Self::Designation => "Designation",
            Self::Place => "Place",
            Self::DateOfSigning => "Date of signing",
        }
    }

    pub fn column(self) -> SodColumn {
        match self {
            Self::NseSymbol => SodColumn::NseSymbol,
            Self::ScripCode => SodColumn::ScripCode,
            Self::MseiSymbol => SodColumn::MseiSymbol,
            Self::Isin => SodColumn::Isin,
            Self::CompanyName => SodColumn::CompanyName,
            Self::StatementCount => SodColumn::StatementCount,
            Self::QuarterEnded => SodColumn::QuarterEnded,
            Self::ModeOfFundRaising => SodColumn::ModeOfFundRaising,
            Self::DateOfFundsRaising => SodColumn::DateOfFundsRaising,
            Self::AmountRaised => SodColumn::AmountRaised,
            Self::MonitoringAgency => SodColumn::MonitoringAgency,
            Self::MonitoringAgencyName => SodColumn::MonitoringAgencyName,
            Self::HasDeviation => SodColumn::HasDeviation,
            Self::DeviationExplanation => SodColumn::DeviationExplanation,
            Self::ShareholderApproved => SodColumn::ShareholderApproved,
            Self::AuditCommitteeComments => SodColumn::AuditCommitteeComments,
            Self::AuditorComments => SodColumn::AuditorComments,
            Self::Signatory => SodColumn::Signatory,
            Self::Designation => SodColumn::Designation,
            Self::Place => SodColumn::Place,
            Self::DateOfSigning => SodColumn::DateOfSigning,
        }
    }

    pub fn display(self, item: &SodModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::Isin => opt_str(&item.isin),
            Self::CompanyName => opt_str(&item.company_name),
            Self::StatementCount => opt_str(&item.statement_count),
            Self::QuarterEnded => opt_date(item.quarter_ended),
            Self::ModeOfFundRaising => opt_str(&item.mode_of_fund_raising),
            Self::DateOfFundsRaising => opt_date(item.date_of_funds_raising),
            Self::AmountRaised => opt_str(&item.amount_raised),
            Self::MonitoringAgency => opt_str(&item.monitoring_agency),
            Self::MonitoringAgencyName => opt_str(&item.monitoring_agency_name),
            Self::HasDeviation => opt_str(&item.has_deviation),
            Self::DeviationExplanation => opt_str(&item.deviation_explanation),
            Self::ShareholderApproved => opt_str(&item.shareholder_approved),
            Self::AuditCommitteeComments => opt_str(&item.audit_committee_comments),
            Self::AuditorComments => opt_str(&item.auditor_comments),
            Self::Signatory => opt_str(&item.signatory),
            Self::Designation => opt_str(&item.designation),
            Self::Place => opt_str(&item.place),
            Self::DateOfSigning => opt_date(item.date_of_signing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SodObjectRow, parse_sod_xbrl};
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="http://www.bseindia.com/xbrl/2021-03-31/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<xbrli:context id="OneI"><xbrli:entity><xbrli:identifier scheme="http://www.nseindia.com/ScripCode">123456</xbrli:identifier></xbrli:entity><xbrli:period><xbrli:instant>2024-12-31</xbrli:instant></xbrli:period></xbrli:context>
<in-capmkt:ScripCode contextRef="OneI">123456</in-capmkt:ScripCode>
<in-capmkt:NSESymbol contextRef="OneI">ARIHANTACA</in-capmkt:NSESymbol>
<in-capmkt:MSEISymbol contextRef="OneI">NOTLISTED</in-capmkt:MSEISymbol>
<in-capmkt:ISIN contextRef="OneI">INE0NCC01015</in-capmkt:ISIN>
<in-capmkt:NameOfTheCompany contextRef="OneI">Arihant Academy Limited</in-capmkt:NameOfTheCompany>
<in-capmkt:NumberOfStatementsOfDeviation contextRef="OneD">1</in-capmkt:NumberOfStatementsOfDeviation>
<in-capmkt:ReportFiledForQuarterEnded contextRef="OneI">2024-12-31</in-capmkt:ReportFiledForQuarterEnded>
<in-capmkt:ModeOfFundRaising contextRef="StatementStatic1I">Public Issues</in-capmkt:ModeOfFundRaising>
<in-capmkt:DateOfFundsRaising contextRef="StatementStatic1I">2022-12-29</in-capmkt:DateOfFundsRaising>
<in-capmkt:AmountRaised contextRef="StatementStatic1I" unitRef="INR" decimals="-3">14.717</in-capmkt:AmountRaised>
<in-capmkt:MonitoringAgency contextRef="StatementStatic1I">Not Applicable</in-capmkt:MonitoringAgency>
<in-capmkt:IsThereADeviationOrVariationInUseOfFundsRaised contextRef="StatementStatic1I">false</in-capmkt:IsThereADeviationOrVariationInUseOfFundsRaised>
<in-capmkt:CommentsOfTheAuditCommitteeAfterReview contextRef="StatementStatic1I">No Comments</in-capmkt:CommentsOfTheAuditCommitteeAfterReview>
<in-capmkt:CommentsOfTheAuditors contextRef="StatementStatic1I">No Comments</in-capmkt:CommentsOfTheAuditors>
<in-capmkt:OriginalObject contextRef="StatementDynamic1I1">Funding
Working Capital Requirements</in-capmkt:OriginalObject>
<in-capmkt:ModifiedObject contextRef="StatementDynamic1I1">NA</in-capmkt:ModifiedObject>
<in-capmkt:OriginalAllocation contextRef="StatementDynamic1I1" unitRef="INR" decimals="-3">110000000</in-capmkt:OriginalAllocation>
<in-capmkt:ModifiedAllocation contextRef="StatementDynamic1I1" unitRef="INR" decimals="-3">0</in-capmkt:ModifiedAllocation>
<in-capmkt:FundsUtilised contextRef="StatementDynamic1I1" unitRef="INR" decimals="-3">95116466</in-capmkt:FundsUtilised>
<in-capmkt:AmountOfDeviationOrVariationForTheQuarterAccordingToApplicableObject contextRef="StatementDynamic1I1" unitRef="INR" decimals="-3">0</in-capmkt:AmountOfDeviationOrVariationForTheQuarterAccordingToApplicableObject>
<in-capmkt:DisclosureNotesOnObjectsForWhichFundsHaveBeenRaisedAndWhereThereHasBeenADeviation contextRef="StatementDynamic1I1">Working capital drawdown</in-capmkt:DisclosureNotesOnObjectsForWhichFundsHaveBeenRaisedAndWhereThereHasBeenADeviation>
<in-capmkt:OriginalObject contextRef="StatementDynamic1I2">General corporate purposes</in-capmkt:OriginalObject>
<in-capmkt:ModifiedObject contextRef="StatementDynamic1I2">NA</in-capmkt:ModifiedObject>
<in-capmkt:OriginalAllocation contextRef="StatementDynamic1I2" unitRef="INR" decimals="-3">27206000</in-capmkt:OriginalAllocation>
<in-capmkt:ModifiedAllocation contextRef="StatementDynamic1I2" unitRef="INR" decimals="-3">0</in-capmkt:ModifiedAllocation>
<in-capmkt:FundsUtilised contextRef="StatementDynamic1I2" unitRef="INR" decimals="-3">27206000</in-capmkt:FundsUtilised>
<in-capmkt:AmountOfDeviationOrVariationForTheQuarterAccordingToApplicableObject contextRef="StatementDynamic1I2" unitRef="INR" decimals="-3">0</in-capmkt:AmountOfDeviationOrVariationForTheQuarterAccordingToApplicableObject>
<in-capmkt:NameOfSignatory contextRef="OneD">ANIL SURESH KAPASI</in-capmkt:NameOfSignatory>
<in-capmkt:DesignationOfPerson contextRef="OneD">Managing Director</in-capmkt:DesignationOfPerson>
<in-capmkt:Place contextRef="OneD">Mumbai</in-capmkt:Place>
<in-capmkt:DateOfSigning contextRef="OneD">2025-02-14</in-capmkt:DateOfSigning>
</xbrli:xbrl>
"#;

    const TWO_STATEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="http://www.bseindia.com/xbrl/2021-03-31/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-capmkt:NSESymbol contextRef="OneI">VAIDYA</in-capmkt:NSESymbol>
<in-capmkt:ISIN contextRef="OneI">INE0JR201016</in-capmkt:ISIN>
<in-capmkt:NameOfTheCompany contextRef="OneI">Vaidya Sane Ayurved Laboratories Limited</in-capmkt:NameOfTheCompany>
<in-capmkt:NumberOfStatementsOfDeviation contextRef="OneD">2</in-capmkt:NumberOfStatementsOfDeviation>
<in-capmkt:ModeOfFundRaising contextRef="StatementStatic1I">Preferential Issues</in-capmkt:ModeOfFundRaising>
<in-capmkt:DateOfFundsRaising contextRef="StatementStatic1I">2024-05-27</in-capmkt:DateOfFundsRaising>
<in-capmkt:AmountRaised contextRef="StatementStatic1I">27668750</in-capmkt:AmountRaised>
<in-capmkt:IsThereADeviationOrVariationInUseOfFundsRaised contextRef="StatementStatic1I">false</in-capmkt:IsThereADeviationOrVariationInUseOfFundsRaised>
<in-capmkt:ModeOfFundRaising contextRef="StatementStatic2I">Preferential Issues</in-capmkt:ModeOfFundRaising>
<in-capmkt:DateOfFundsRaising contextRef="StatementStatic2I">2023-08-14</in-capmkt:DateOfFundsRaising>
<in-capmkt:AmountRaised contextRef="StatementStatic2I">106644600</in-capmkt:AmountRaised>
<in-capmkt:IsThereADeviationOrVariationInUseOfFundsRaised contextRef="StatementStatic2I">true</in-capmkt:IsThereADeviationOrVariationInUseOfFundsRaised>
<in-capmkt:ExplanationForTheDeviationOrVariation contextRef="StatementStatic2I">Reallocation within objects</in-capmkt:ExplanationForTheDeviationOrVariation>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_identity_statement_and_objects() {
        let facts = parse_sod_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("ARIHANTACA"));
        assert_eq!(facts.scrip_code.as_deref(), Some("123456"));
        assert_eq!(facts.isin.as_deref(), Some("INE0NCC01015"));
        assert_eq!(
            facts.company_name.as_deref(),
            Some("Arihant Academy Limited")
        );
        assert_eq!(facts.statement_count.as_deref(), Some("1"));
        assert_eq!(facts.quarter_ended, NaiveDate::from_ymd_opt(2024, 12, 31));
        assert_eq!(facts.mode_of_fund_raising.as_deref(), Some("Public Issues"));
        assert_eq!(
            facts.date_of_funds_raising,
            NaiveDate::from_ymd_opt(2022, 12, 29)
        );
        assert_eq!(facts.amount_raised.as_deref(), Some("14.717"));
        assert_eq!(facts.monitoring_agency.as_deref(), Some("Not Applicable"));
        assert_eq!(facts.has_deviation.as_deref(), Some("false"));
        assert_eq!(facts.signatory.as_deref(), Some("ANIL SURESH KAPASI"));
        assert_eq!(facts.designation.as_deref(), Some("Managing Director"));
        assert_eq!(facts.place.as_deref(), Some("Mumbai"));
        assert_eq!(facts.date_of_signing, NaiveDate::from_ymd_opt(2025, 2, 14));
        assert_eq!(
            facts.objects,
            vec![
                SodObjectRow {
                    object: "Funding Working Capital Requirements".into(),
                    modified_object: "NA".into(),
                    original_allocation: "110000000".into(),
                    modified_allocation: "0".into(),
                    funds_utilised: "95116466".into(),
                    amount_of_deviation: "0".into(),
                    notes: "Working capital drawdown".into(),
                },
                SodObjectRow {
                    object: "General corporate purposes".into(),
                    modified_object: "NA".into(),
                    original_allocation: "27206000".into(),
                    modified_allocation: "0".into(),
                    funds_utilised: "27206000".into(),
                    amount_of_deviation: "0".into(),
                    notes: String::new(),
                },
            ]
        );
    }

    #[test]
    fn joins_multiple_statements_and_any_deviation() {
        let facts = parse_sod_xbrl(TWO_STATEMENTS);
        assert_eq!(facts.statement_count.as_deref(), Some("2"));
        assert_eq!(
            facts.mode_of_fund_raising.as_deref(),
            Some("Preferential Issues | Preferential Issues")
        );
        assert_eq!(facts.amount_raised.as_deref(), Some("27668750 | 106644600"));
        assert_eq!(facts.has_deviation.as_deref(), Some("true"));
        assert_eq!(
            facts.deviation_explanation.as_deref(),
            Some("Reallocation within objects")
        );
        assert_eq!(facts.objects, vec![]);
        assert_eq!(
            facts.date_of_funds_raising,
            NaiveDate::from_ymd_opt(2024, 5, 27)
        );
    }

    #[test]
    fn object_rows_roundtrip_json() {
        let facts = parse_sod_xbrl(FIXTURE);
        let json = serde_json::to_value(&facts.objects).unwrap();
        let back: Vec<SodObjectRow> = serde_json::from_value(json).unwrap();
        assert_eq!(back, facts.objects);
        let cols = super::object_table_columns(&facts.objects);
        let headers: Vec<_> = cols.iter().map(|c| c.header).collect();
        assert_eq!(
            headers,
            [
                "Object",
                "Modified",
                "Original allocation",
                "Modified allocation",
                "Funds utilised",
                "Amount of deviation",
                "Notes",
            ]
        );
    }
}
