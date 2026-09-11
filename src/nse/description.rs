//! Typed fields encoded in NSE RSS `<description>` text (not XML schema).

use chrono::{DateTime, NaiveDate, Utc};
use lariv_rs::datetime::{DatetimeLabel, format_date};

pub use crate::dates::{parse_date as parse_nse_date, parse_datetime as parse_nse_datetime};

macro_rules! field_ty {
    (text) => {
        Option<String>
    };
    (date) => {
        Option<NaiveDate>
    };
    (datetime) => {
        Option<DateTime<Utc>>
    };
}

macro_rules! assign_kind {
    (text, $slot:expr, $value:expr) => {{
        if $slot.is_none() {
            $slot = Some($value);
        }
        true
    }};
    (date, $slot:expr, $value:expr) => {{
        if is_blank_date(&$value) {
            true
        } else if let Some(d) = parse_nse_date(&$value) {
            if $slot.is_none() {
                $slot = Some(d);
            }
            true
        } else {
            false
        }
    }};
    (datetime, $slot:expr, $value:expr) => {{
        if is_blank_date(&$value) {
            true
        } else if let Some(d) = parse_nse_datetime(&$value) {
            if $slot.is_none() {
                $slot = Some(d);
            }
            true
        } else {
            false
        }
    }};
}

macro_rules! value_kind_of {
    (text) => {
        crate::list_filters::ValueKind::Text
    };
    (date) => {
        crate::list_filters::ValueKind::Date
    };
    (datetime) => {
        crate::list_filters::ValueKind::DateTime
    };
}

macro_rules! display_kind {
    (text, $val:expr, $tz:expr) => {
        $val.clone().unwrap_or_default()
    };
    (date, $val:expr, $tz:expr) => {
        $val.map(format_date).unwrap_or_default()
    };
    (datetime, $val:expr, $tz:expr) => {
        $val.map(|dt| DatetimeLabel::seconds(dt, $tz).into_string())
            .unwrap_or_default()
    };
}

macro_rules! nse_desc_fields {
    ($(
        $variant:ident => {
            sort: $sort:expr,
            label: $label:expr,
            nse_key: $nse:expr,
            model: $model_field:ident,
            kind: $kind:ident,
        }
    ),* $(,)?) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub enum DescriptionField {
            $($variant,)*
        }

        impl DescriptionField {
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            pub fn sort_key(self) -> &'static str {
                match self {
                    $(Self::$variant => $sort,)*
                }
            }

            pub fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)*
                }
            }

            pub fn nse_key(self) -> &'static str {
                match self {
                    $(Self::$variant => $nse,)*
                }
            }

            pub fn from_nse_key(key: &str) -> Option<Self> {
                let n = normalize_key(key);
                $(
                    if n == normalize_key($nse) {
                        return Some(Self::$variant);
                    }
                )*
                None
            }

            pub fn display(self, fields: &NseDescriptionFields, tz: &str) -> String {
                match self {
                    $(Self::$variant => display_kind!($kind, fields.$model_field, tz),)*
                }
            }

            pub fn value_kind(self) -> crate::list_filters::ValueKind {
                match self {
                    $(Self::$variant => value_kind_of!($kind),)*
                }
            }

            fn assign(self, fields: &mut NseDescriptionFields, value: String) -> bool {
                fields.keys_extracted = true;
                match self {
                    $(Self::$variant => assign_kind!($kind, fields.$model_field, value),)*
                }
            }
        }

        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct NseDescriptionFields {
            pub remainder: String,
            keys_extracted: bool,
            $(pub $model_field: field_ty!($kind),)*
        }

        impl NseDescriptionFields {
            pub fn any_extracted(&self) -> bool {
                self.keys_extracted
            }

            pub fn inferred_pub_date(&self, rss_pub_date: &str) -> Option<DateTime<Utc>> {
                crate::dates::parse_datetime(rss_pub_date)
                    .or(self.original_submission_date)
                    .or_else(|| self.as_on_date.and_then(crate::dates::date_start_ist))
            }
        }
    };
}

nse_desc_fields! {
    Subject => {
        sort: "Subject",
        label: "Subject",
        nse_key: "SUBJECT",
        model: subject,
        kind: text,
    },
    AsOnDate => {
        sort: "AsOnDate",
        label: "As on date",
        nse_key: "AS ON DATE",
        model: as_on_date,
        kind: date,
    },
    OriginalSubmissionDate => {
        sort: "OriginalSubmissionDate",
        label: "Original submission date",
        nse_key: "ORIGINAL SUBMISSION DATE",
        model: original_submission_date,
        kind: datetime,
    },
    Series => {
        sort: "Series",
        label: "Series",
        nse_key: "SERIES",
        model: series,
        kind: text,
    },
    Purpose => {
        sort: "Purpose",
        label: "Purpose",
        nse_key: "PURPOSE",
        model: purpose,
        kind: text,
    },
    FaceValue => {
        sort: "FaceValue",
        label: "Face value",
        nse_key: "FACE VALUE",
        model: face_value,
        kind: text,
    },
    RecordDate => {
        sort: "RecordDate",
        label: "Record date",
        nse_key: "RECORD DATE",
        model: record_date,
        kind: date,
    },
    BookClosureStartDate => {
        sort: "BookClosureStartDate",
        label: "Book closure start date",
        nse_key: "BOOK CLOSURE START DATE",
        model: book_closure_start_date,
        kind: date,
    },
    BookClosureEndDate => {
        sort: "BookClosureEndDate",
        label: "Book closure end date",
        nse_key: "BOOK CLOSURE END DATE",
        model: book_closure_end_date,
        kind: date,
    },
    RelatingTo => {
        sort: "RelatingTo",
        label: "Relating to",
        nse_key: "RELATING TO",
        model: relating_to,
        kind: text,
    },
    AuditedUnaudited => {
        sort: "AuditedUnaudited",
        label: "Audited/Unaudited",
        nse_key: "AUDITED/UNAUDITED",
        model: audited_unaudited,
        kind: text,
    },
    Cumulative => {
        sort: "Cumulative",
        label: "Cumulative/Non-cumulative",
        nse_key: "CUMULATIVE/NON-CUMULATIVE",
        model: cumulative,
        kind: text,
    },
    Consolidated => {
        sort: "Consolidated",
        label: "Consolidated/Non-consolidated",
        nse_key: "CONSOLIDATED/NON-CONSOLIDATED",
        model: consolidated,
        kind: text,
    },
    IndAs => {
        sort: "IndAs",
        label: "Ind AS",
        nse_key: "IND AS/ NON IND AS",
        model: ind_as,
        kind: text,
    },
    Period => {
        sort: "Period",
        label: "Period",
        nse_key: "PERIOD",
        model: period,
        kind: text,
    },
    PeriodEnded => {
        sort: "PeriodEnded",
        label: "Period ended",
        nse_key: "PERIOD ENDED",
        model: period_ended,
        kind: date,
    },
    ForQuarterEnding => {
        sort: "ForQuarterEnding",
        label: "For quarter ending",
        nse_key: "FOR QUARTER ENDING",
        model: for_quarter_ending,
        kind: date,
    },
    EncumberedPromoterNames => {
        sort: "EncumberedPromoterNames",
        label: "Encumbered promoter(s)",
        nse_key: "NAME OF THE PROMOTER(S) / PACS WHOSE SHARES HAVE BEEN ENCUMBERED",
        model: encumbered_promoter_names,
        kind: text,
    },
    PeriodEndDate => {
        sort: "PeriodEndDate",
        label: "Period end date",
        nse_key: "PERIOD END DATE",
        model: period_end_date,
        kind: date,
    },
    AcquirerNames => {
        sort: "AcquirerNames",
        label: "Acquirer / PAC",
        nse_key: "NAME(S)OF THE ACQUIRER AND ITS(PAC)",
        model: acquirer_names,
        kind: text,
    },
    PromoterNames => {
        sort: "PromoterNames",
        label: "Promoter(s) / PAC",
        nse_key: "NAME OF PROMOTER(S) OR PACS WITH HIM",
        model: promoter_names,
        kind: text,
    },
    FinancialYear => {
        sort: "FinancialYear",
        label: "Financial year",
        nse_key: "FINANCIAL YEAR",
        model: financial_year,
        kind: text,
    },
    SubmissionType => {
        sort: "SubmissionType",
        label: "Submission type",
        nse_key: "SUBMISSION TYPE",
        model: submission_type,
        kind: text,
    },
    MeetingDate => {
        sort: "MeetingDate",
        label: "Meeting date",
        nse_key: "MEETING DATE",
        model: meeting_date,
        kind: date,
    },
    Remarks => {
        sort: "Remarks",
        label: "Remarks",
        nse_key: "REMARKS",
        model: remarks,
        kind: text,
    },
}

fn is_blank_date(s: &str) -> bool {
    crate::dates::is_blank(s)
}

fn normalize_key(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_uppercase()
        .replace(" /", "/")
        .replace("/ ", "/")
}

fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .to_ascii_uppercase()
        .find(&needle.to_ascii_uppercase())
        .filter(|&i| haystack.is_char_boundary(i))
}

fn find_known_key(s: &str) -> Option<(usize, DescriptionField)> {
    let mut best: Option<(usize, usize, DescriptionField)> = None;
    for field in DescriptionField::ALL {
        if *field == DescriptionField::Remarks {
            continue;
        }
        let key = field.nse_key();
        let mut from = 0;
        while from < s.len() {
            let Some(rel) = find_ci(&s[from..], key) else {
                break;
            };
            let idx = from + rel;
            let after = idx + key.len();
            if after <= s.len() && s.is_char_boundary(after) {
                let trimmed = s[after..].trim_start();
                if trimmed.starts_with(':') {
                    let keylen = key.len();
                    let better = match best {
                        None => true,
                        Some((bidx, blen, _)) => idx < bidx || (idx == bidx && keylen > blen),
                    };
                    if better {
                        best = Some((idx, keylen, *field));
                    }
                    break;
                }
            }
            from = idx + 1;
            while from < s.len() && !s.is_char_boundary(from) {
                from += 1;
            }
        }
    }
    best.map(|(idx, _, field)| (idx, field))
}

fn split_next_known_key(value: &str) -> (String, Option<&str>) {
    let mut cut: Option<usize> = None;
    for field in DescriptionField::ALL {
        if *field == DescriptionField::Remarks {
            continue;
        }
        let needle = format!(", {}", field.nse_key());
        if let Some(i) = find_ci(value, &needle)
            && cut.map(|c| i < c).unwrap_or(true)
        {
            cut = Some(i);
        }
    }
    match cut {
        Some(i) => (value[..i].to_string(), Some(value[i + 1..].trim())),
        None => (value.to_string(), None),
    }
}

fn parse_segment(part: &str, fields: &mut NseDescriptionFields) -> Option<String> {
    let trimmed = part.trim();
    if trimmed.is_empty() {
        return None;
    }
    let Some((idx, field)) = find_known_key(trimmed) else {
        return Some(trimmed.to_string());
    };
    let before = trimmed[..idx]
        .trim_end_matches(|c: char| c.is_whitespace() || c == '|')
        .trim();
    let after_key = trimmed[idx + field.nse_key().len()..].trim_start();
    let Some(after_colon) = after_key.strip_prefix(':') else {
        return Some(trimmed.to_string());
    };
    let after_colon = after_colon.trim_start();
    let (value, rest) = split_next_known_key(after_colon);
    let value = value.trim();
    let mut leftover = Vec::new();
    if !before.is_empty() {
        leftover.push(before.to_string());
    }
    if !value.is_empty() && !field.assign(fields, value.to_string()) {
        leftover.push(value.to_string());
    } else if value.is_empty() {
        fields.keys_extracted = true;
    }
    if let Some(rest) = rest
        && let Some(more) = parse_segment(rest, fields)
    {
        leftover.push(more);
    }
    if leftover.is_empty() {
        None
    } else {
        Some(leftover.join(" | "))
    }
}

fn apply_integrated_filing(leftover: &mut Vec<String>, fields: &mut NseDescriptionFields) {
    const MARKER: &str = "Integrated Filing- Financials";
    if leftover
        .first()
        .is_none_or(|s| !s.eq_ignore_ascii_case(MARKER))
    {
        return;
    }
    leftover.remove(0);
    fields.keys_extracted = true;
    if leftover
        .first()
        .is_some_and(|s| s.eq_ignore_ascii_case("Original") || s.eq_ignore_ascii_case("Revision"))
    {
        DescriptionField::SubmissionType.assign(fields, leftover.remove(0));
    }
    let remarks = leftover
        .drain(..)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    if !remarks.is_empty() {
        DescriptionField::Remarks.assign(fields, remarks);
    }
}

/// Pull NSE `KEY: value` (and Integrated Filing positional) encodings out of `<description>`.
pub fn parse_nse_description(description: &str) -> NseDescriptionFields {
    let mut fields = NseDescriptionFields::default();
    let mut leftover = Vec::new();
    for part in description.split('|') {
        if let Some(rest) = parse_segment(part, &mut fields) {
            leftover.push(rest);
        }
    }
    apply_integrated_filing(&mut leftover, &mut fields);
    fields.remainder = leftover
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    fields
}

#[cfg(test)]
mod tests {
    use super::{parse_nse_date, parse_nse_datetime, parse_nse_description};
    use chrono::{NaiveDate, TimeZone, Timelike, Utc};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn parses_two_and_four_digit_dates() {
        assert_eq!(parse_nse_date("07-SEP-26"), Some(date(2026, 9, 7)));
        assert_eq!(parse_nse_date("18-Sep-2026"), Some(date(2026, 9, 18)));
        assert_eq!(parse_nse_date("31-Mar-2021"), Some(date(2021, 3, 31)));
        assert_eq!(parse_nse_date("-"), None);
        assert_eq!(parse_nse_date(""), None);
    }

    #[test]
    fn parses_ist_datetime_to_utc() {
        let dt = parse_nse_datetime("07-SEP-26 11.26.38.383800 PM").unwrap();
        assert_eq!(
            dt,
            Utc.with_ymd_and_hms(2026, 9, 7, 17, 56, 38)
                .unwrap()
                .with_nanosecond(383_800_000)
                .unwrap()
        );
        let pub_dt = parse_nse_datetime("07-Sep-2026 23:28:05").unwrap();
        assert_eq!(pub_dt, Utc.with_ymd_and_hms(2026, 9, 7, 17, 58, 5).unwrap());
        let rfc = parse_nse_datetime("Mon, 7 Sep 2026 00:00:00 +0530").unwrap();
        assert_eq!(rfc, Utc.with_ymd_and_hms(2026, 9, 6, 18, 30, 0).unwrap());
        let iso = parse_nse_datetime("2026-09-07T17:58:05+00:00").unwrap();
        assert_eq!(iso, Utc.with_ymd_and_hms(2026, 9, 7, 12, 28, 5).unwrap());
    }

    #[test]
    fn splits_annual_report_as_on_date() {
        let parsed = parse_nse_description("AS ON DATE : 07-SEP-26");
        assert_eq!(parsed.remainder, "");
        assert_eq!(parsed.as_on_date, Some(date(2026, 9, 7)));
    }

    #[test]
    fn splits_as_on_date_from_mixed_description() {
        let parsed = parse_nse_description("Annual report filed | AS ON DATE : 08-SEP-26 | extra");
        assert_eq!(parsed.remainder, "Annual report filed | extra");
        assert_eq!(parsed.as_on_date, Some(date(2026, 9, 8)));
    }

    #[test]
    fn splits_announcement_subject() {
        let parsed = parse_nse_description(
            "Ajax Engineering Limited has informed the Exchange about Resignation of Director/KMP/SMP |SUBJECT: Resignation of Director/KMP/SMP",
        );
        assert_eq!(
            parsed.remainder,
            "Ajax Engineering Limited has informed the Exchange about Resignation of Director/KMP/SMP"
        );
        assert_eq!(
            parsed.subject.as_deref(),
            Some("Resignation of Director/KMP/SMP")
        );
    }

    #[test]
    fn splits_corporate_actions() {
        let parsed = parse_nse_description(
            "SERIES:EQ |PURPOSE:DIVIDEND - RS 4 PER SHARE |FACE VALUE:2 |RECORD DATE:18-Sep-2026 |BOOK CLOSURE START DATE:- |BOOK CLOSURE END DATE:-",
        );
        assert_eq!(parsed.remainder, "");
        assert_eq!(parsed.series.as_deref(), Some("EQ"));
        assert_eq!(parsed.purpose.as_deref(), Some("DIVIDEND - RS 4 PER SHARE"));
        assert_eq!(parsed.face_value.as_deref(), Some("2"));
        assert_eq!(parsed.record_date, Some(date(2026, 9, 18)));
        assert_eq!(parsed.book_closure_start_date, None);
        assert_eq!(parsed.book_closure_end_date, None);
    }

    #[test]
    fn splits_financial_results_without_period_stealing_period_ended() {
        let parsed = parse_nse_description(
            "RELATING TO:Annual |AUDITED/UNAUDITED:Audited |CUMULATIVE/NON-CUMULATIVE:Cumulative |CONSOLIDATED/NON-CONSOLIDATED:Consolidated |IND AS/ NON IND AS:Ind-AS New |PERIOD:Annual |PERIOD ENDED: 31-Mar-2021",
        );
        assert_eq!(parsed.remainder, "");
        assert_eq!(parsed.relating_to.as_deref(), Some("Annual"));
        assert_eq!(parsed.period.as_deref(), Some("Annual"));
        assert_eq!(parsed.period_ended, Some(date(2021, 3, 31)));
    }

    #[test]
    fn splits_secretarial_comma_separated_keys() {
        let parsed = parse_nse_description(
            "FINANCIAL YEAR : Apr-2024 to Mar-2025, SUBMISSION TYPE: Revision",
        );
        assert_eq!(parsed.remainder, "");
        assert_eq!(
            parsed.financial_year.as_deref(),
            Some("Apr-2024 to Mar-2025")
        );
        assert_eq!(parsed.submission_type.as_deref(), Some("Revision"));
    }

    #[test]
    fn splits_integrated_filing_positional() {
        let parsed = parse_nse_description(
            "Integrated Filing- Financials|Revision|covering letter clerical error",
        );
        assert_eq!(parsed.remainder, "");
        assert_eq!(parsed.submission_type.as_deref(), Some("Revision"));
        assert_eq!(
            parsed.remarks.as_deref(),
            Some("covering letter clerical error")
        );
    }

    #[test]
    fn splits_integrated_filing_original_empty_remarks() {
        let parsed = parse_nse_description("Integrated Filing- Financials|Original|");
        assert_eq!(parsed.remainder, "");
        assert_eq!(parsed.submission_type.as_deref(), Some("Original"));
        assert!(parsed.remarks.is_none());
    }

    #[test]
    fn leaves_unstructured_offer_documents() {
        let original = "Deepa Jewellers Limited has filled PROSP for its IPO";
        let parsed = parse_nse_description(original);
        assert_eq!(parsed.remainder, original);
        assert!(!parsed.any_extracted());
    }

    #[test]
    fn splits_brsr_and_voting_and_encumbrance() {
        let brsr = parse_nse_description("ORIGINAL SUBMISSION DATE : 07-SEP-26 11.26.38.383800 PM");
        assert_eq!(
            brsr.original_submission_date,
            parse_nse_datetime("07-SEP-26 11.26.38.383800 PM")
        );
        let vote = parse_nse_description("MEETING DATE : 07-SEP-2026");
        assert_eq!(vote.meeting_date, Some(date(2026, 9, 7)));
        let enc = parse_nse_description(
            "NAME OF THE PROMOTER(S) / PACS WHOSE SHARES HAVE BEEN ENCUMBERED : 1. M/s Dream Home Developers Pvt. Ltd.",
        );
        assert_eq!(
            enc.encumbered_promoter_names.as_deref(),
            Some("1. M/s Dream Home Developers Pvt. Ltd.")
        );
        assert_eq!(enc.inferred_pub_date(""), None);
    }

    #[test]
    fn infers_pub_date_from_empty_rss_tag() {
        let annual = parse_nse_description("AS ON DATE : 08-SEP-26");
        assert_eq!(
            annual.inferred_pub_date(""),
            parse_nse_datetime("08-SEP-26")
        );
        let brsr = parse_nse_description("ORIGINAL SUBMISSION DATE : 07-SEP-26 11.26.38.383800 PM");
        assert_eq!(
            brsr.inferred_pub_date(""),
            parse_nse_datetime("07-SEP-26 11.26.38.383800 PM")
        );
        let rss_wins = parse_nse_description("AS ON DATE : 08-SEP-26");
        assert_eq!(
            rss_wins.inferred_pub_date("07-Sep-2026 23:28:05"),
            parse_nse_datetime("07-Sep-2026 23:28:05")
        );
    }
}
