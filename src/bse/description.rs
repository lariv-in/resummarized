//! Typed fields encoded in BSE RSS `<description>` text (not XML schema).

use chrono::{DateTime, NaiveDate, Utc};
#[allow(unused_imports)]
use lariv_rs::datetime::{DatetimeLabel, format_date};

pub use crate::dates::{parse_date as parse_bse_date, parse_datetime as parse_bse_datetime};

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
        let value = $value.trim();
        if value.is_empty() || value == "-" {
            true
        } else if $slot.is_none() {
            $slot = Some(value.to_string());
            true
        } else {
            true
        }
    }};
    (date, $slot:expr, $value:expr) => {{
        if is_blank_date(&$value) {
            true
        } else if let Some(d) = parse_bse_date(&$value) {
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
        } else if let Some(d) = parse_bse_datetime(&$value) {
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
    (text, $val:expr, $tz:expr) => {{
        let _ = $tz;
        $val.clone().unwrap_or_default()
    }};
    (date, $val:expr, $tz:expr) => {{
        let _ = $tz;
        $val.map(format_date).unwrap_or_default()
    }};
    (datetime, $val:expr, $tz:expr) => {
        $val.map(|dt| DatetimeLabel::seconds(dt, $tz).into_string())
            .unwrap_or_default()
    };
}

macro_rules! bse_desc_fields {
    ($(
        $variant:ident => {
            sort: $sort:expr,
            label: $label:expr,
            bse_key: $bse:expr,
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

            pub fn bse_key(self) -> &'static str {
                match self {
                    $(Self::$variant => $bse,)*
                }
            }

            pub fn from_bse_key(key: &str) -> Option<Self> {
                let n = normalize_key(key);
                $(
                    if n == normalize_key($bse) {
                        return Some(Self::$variant);
                    }
                )*
                None
            }

            pub fn display(self, fields: &BseDescriptionFields, tz: &str) -> String {
                match self {
                    $(Self::$variant => display_kind!($kind, fields.$model_field, tz),)*
                }
            }

            pub fn value_kind(self) -> crate::list_filters::ValueKind {
                match self {
                    $(Self::$variant => value_kind_of!($kind),)*
                }
            }

            fn assign(self, fields: &mut BseDescriptionFields, value: String) -> bool {
                fields.keys_extracted = true;
                match self {
                    $(Self::$variant => assign_kind!($kind, fields.$model_field, value),)*
                }
            }
        }

        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct BseDescriptionFields {
            pub remainder: String,
            keys_extracted: bool,
            $(pub $model_field: field_ty!($kind),)*
        }

        impl BseDescriptionFields {
            pub fn any_extracted(&self) -> bool {
                self.keys_extracted
            }

            pub fn inferred_pub_date(&self, rss_pub_date: &str) -> Option<DateTime<Utc>> {
                crate::dates::parse_datetime(rss_pub_date)
                    .or_else(|| self.as_on_date.and_then(crate::dates::date_start_ist))
                    .or_else(|| self.submission_date.and_then(crate::dates::date_start_ist))
            }
        }
    };
}

bse_desc_fields! {
    Scripcode => {
        sort: "Scripcode",
        label: "Scripcode",
        bse_key: "SCRIPCODE",
        model: scripcode,
        kind: text,
    },
    AsOnDate => {
        sort: "AsOnDate",
        label: "As on date",
        bse_key: "AS ON DATE",
        model: as_on_date,
        kind: date,
    },
    MeetingDate => {
        sort: "MeetingDate",
        label: "Meeting date",
        bse_key: "MEETING DATE",
        model: meeting_date,
        kind: date,
    },
    MeetingType => {
        sort: "MeetingType",
        label: "Meeting type",
        bse_key: "MEETING TYPE",
        model: meeting_type,
        kind: text,
    },
    Purpose => {
        sort: "Purpose",
        label: "Purpose",
        bse_key: "PURPOSE",
        model: purpose,
        kind: text,
    },
    Segment => {
        sort: "Segment",
        label: "Segment",
        bse_key: "SEGMENT",
        model: segment,
        kind: text,
    },
    RdDate => {
        sort: "RdDate",
        label: "Record date",
        bse_key: "RD DATE",
        model: rd_date,
        kind: date,
    },
    BcStartDate => {
        sort: "BcStartDate",
        label: "BC start date",
        bse_key: "BC START DATE",
        model: bc_start_date,
        kind: date,
    },
    BcEndDate => {
        sort: "BcEndDate",
        label: "BC end date",
        bse_key: "BC END DATE",
        model: bc_end_date,
        kind: date,
    },
    NdStartDate => {
        sort: "NdStartDate",
        label: "ND start date",
        bse_key: "ND START DATE",
        model: nd_start_date,
        kind: date,
    },
    NdEndDate => {
        sort: "NdEndDate",
        label: "ND end date",
        bse_key: "ND END DATE",
        model: nd_end_date,
        kind: date,
    },
    ActualPaymentDate => {
        sort: "ActualPaymentDate",
        label: "Actual payment date",
        bse_key: "ACTUAL PAYMENT DATE",
        model: actual_payment_date,
        kind: date,
    },
    TypeOfSecurity => {
        sort: "TypeOfSecurity",
        label: "Type of security",
        bse_key: "TYPE OF SECURITY",
        model: type_of_security,
        kind: text,
    },
    AuditedUnaudited => {
        sort: "AuditedUnaudited",
        label: "Audited / unaudited",
        bse_key: "AUDITED/UNAUDITED",
        model: audited_unaudited,
        kind: text,
    },
    StandaloneConsolidated => {
        sort: "StandaloneConsolidated",
        label: "Standalone / consolidated",
        bse_key: "STANDALONE/CONSOLIDATED",
        model: standalone_consolidated,
        kind: text,
    },
    PeriodStartDate => {
        sort: "PeriodStartDate",
        label: "Period start date",
        bse_key: "PERIOD START DATE",
        model: period_start_date,
        kind: date,
    },
    PeriodEndDate => {
        sort: "PeriodEndDate",
        label: "Period end date",
        bse_key: "PERIOD END DATE",
        model: period_end_date,
        kind: date,
    },
    IndAs => {
        sort: "IndAs",
        label: "Ind AS",
        bse_key: "IND AS/NON IND AS",
        model: ind_as,
        kind: text,
    },
    PromoterAndGroup => {
        sort: "PromoterAndGroup",
        label: "Promoter and group",
        bse_key: "PR_AND_PRGRP",
        model: promoter_and_group,
        kind: text,
    },
    PublicVal => {
        sort: "PublicVal",
        label: "Public",
        bse_key: "PUBLIC_VAL",
        model: public_val,
        kind: text,
    },
    Emptr => {
        sort: "Emptr",
        label: "Employee trust",
        bse_key: "EMPTR",
        model: emptr,
        kind: text,
    },
    Status => {
        sort: "Status",
        label: "Status",
        bse_key: "STATUS",
        model: status,
        kind: text,
    },
    SubmissionDate => {
        sort: "SubmissionDate",
        label: "Submission date",
        bse_key: "SUBMISSION_DT",
        model: submission_date,
        kind: date,
    },
    RevisedFilingDate => {
        sort: "RevisedFilingDate",
        label: "Revised filing date",
        bse_key: "REVISED FILING DATE",
        model: revised_filing_date,
        kind: date,
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
        .replace('_', " ")
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
        if *field == DescriptionField::Scripcode {
            continue;
        }
        let key = field.bse_key();
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
        if *field == DescriptionField::Scripcode {
            continue;
        }
        let needle = format!(", {}", field.bse_key());
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

fn parse_segment(part: &str, fields: &mut BseDescriptionFields) -> Option<String> {
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
    let after_key = trimmed[idx + field.bse_key().len()..].trim_start();
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

fn apply_financial_qualifiers(leftover: &mut Vec<String>, fields: &mut BseDescriptionFields) {
    leftover.retain(|s| {
        let t = s.trim();
        if t.eq_ignore_ascii_case("Audited") || t.eq_ignore_ascii_case("Unaudited") {
            DescriptionField::AuditedUnaudited.assign(fields, t.to_string());
            return false;
        }
        if t.eq_ignore_ascii_case("Standalone") || t.eq_ignore_ascii_case("Consolidated") {
            DescriptionField::StandaloneConsolidated.assign(fields, t.to_string());
            return false;
        }
        true
    });
}

/// Pull BSE `KEY: value` encodings out of `<description>`.
pub fn parse_bse_description(description: &str) -> BseDescriptionFields {
    let mut fields = BseDescriptionFields::default();
    let mut leftover = Vec::new();
    for part in description.split('|') {
        if let Some(rest) = parse_segment(part, &mut fields) {
            leftover.push(rest);
        }
    }
    apply_financial_qualifiers(&mut leftover, &mut fields);
    fields.remainder = leftover
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    fields
}

#[cfg(test)]
mod tests {
    use super::{parse_bse_date, parse_bse_datetime, parse_bse_description};
    use chrono::{NaiveDate, TimeZone, Utc};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn parses_bse_date_formats() {
        assert_eq!(parse_bse_date("08-Sep-2026"), Some(date(2026, 9, 8)));
        assert_eq!(parse_bse_date("08-09-2026"), Some(date(2026, 9, 8)));
        assert_eq!(parse_bse_date("31-03-2026"), Some(date(2026, 3, 31)));
        assert_eq!(parse_bse_date("08 Sep, 2026"), Some(date(2026, 9, 8)));
        assert_eq!(parse_bse_date("Sep 06, 2026"), Some(date(2026, 9, 6)));
        assert_eq!(parse_bse_date("9/8/2026"), Some(date(2026, 9, 8)));
        assert_eq!(parse_bse_date("09/08/2026"), Some(date(2026, 9, 8)));
        assert_eq!(parse_bse_date("-"), None);
        assert_eq!(parse_bse_date(""), None);
    }

    #[test]
    fn parses_ist_and_gmt_datetimes() {
        let pub_dt = parse_bse_datetime("08-Sep-2026 13:06:54").unwrap();
        assert_eq!(pub_dt, Utc.with_ymd_and_hms(2026, 9, 8, 7, 36, 54).unwrap());
        let sensex = parse_bse_datetime("08-09-2026 13:07:49").unwrap();
        assert_eq!(sensex, Utc.with_ymd_and_hms(2026, 9, 8, 7, 37, 49).unwrap());
        let rfc = parse_bse_datetime("Tue, 08 Sep 2026 07:32:03 GMT").unwrap();
        assert_eq!(rfc, Utc.with_ymd_and_hms(2026, 9, 8, 2, 2, 3).unwrap());
        let us = parse_bse_datetime("9/8/2026 2:46:53 PM").unwrap();
        assert_eq!(us, Utc.with_ymd_and_hms(2026, 9, 8, 9, 16, 53).unwrap());
        let us_pad = parse_bse_datetime("09/08/2026 02:46:53 PM").unwrap();
        assert_eq!(us_pad, us);
    }

    #[test]
    fn parses_corporate_actions_description() {
        let parsed = parse_bse_description(
            "SEGMENT : EQUITY | PURPOSE : Final Dividend - Rs. - 0.0100 | RD Date : 08 Sep, 2026 | BC START DATE : -  | BC END DATE : -  | ND START DATE : 08 Sep, 2026 | ND END DATE : 08 Sep, 2026 | Actual Payment Date : - ",
        );
        assert_eq!(parsed.segment.as_deref(), Some("EQUITY"));
        assert_eq!(
            parsed.purpose.as_deref(),
            Some("Final Dividend - Rs. - 0.0100")
        );
        assert_eq!(parsed.rd_date, Some(date(2026, 9, 8)));
        assert_eq!(parsed.bc_start_date, None);
        assert_eq!(parsed.nd_start_date, Some(date(2026, 9, 8)));
        assert_eq!(parsed.actual_payment_date, None);
        assert!(parsed.remainder.is_empty());
    }

    #[test]
    fn parses_financial_results_description() {
        let parsed = parse_bse_description(
            "Audited | Standalone|PERIOD START DATE : 01-01-2026 | PERIOD END DATE : 31-03-2026|IND AS/NON IND AS : IND_AS",
        );
        assert_eq!(parsed.audited_unaudited.as_deref(), Some("Audited"));
        assert_eq!(
            parsed.standalone_consolidated.as_deref(),
            Some("Standalone")
        );
        assert_eq!(parsed.period_start_date, Some(date(2026, 1, 1)));
        assert_eq!(parsed.period_end_date, Some(date(2026, 3, 31)));
        assert_eq!(parsed.ind_as.as_deref(), Some("IND_AS"));
        assert!(parsed.remainder.is_empty());
    }

    #[test]
    fn parses_shareholding_and_voting() {
        let shp = parse_bse_description(
            "AS ON DATE: 31-03-2026 | PR_AND_PRGRP: 69.78 | PUBLIC_VAL: 30.22 | EMPTR:  | STATUS: Revised | SUBMISSION_DT: - | REVISED FILING DATE: 08-09-2026",
        );
        assert_eq!(shp.as_on_date, Some(date(2026, 3, 31)));
        assert_eq!(shp.promoter_and_group.as_deref(), Some("69.78"));
        assert_eq!(shp.public_val.as_deref(), Some("30.22"));
        assert_eq!(shp.status.as_deref(), Some("Revised"));
        assert_eq!(shp.submission_date, None);
        assert_eq!(shp.revised_filing_date, Some(date(2026, 9, 8)));

        let vote =
            parse_bse_description("MEETING DATE : Sep 06, 2026 | MEETING TYPE : Postal Ballot");
        assert_eq!(vote.meeting_date, Some(date(2026, 9, 6)));
        assert_eq!(vote.meeting_type.as_deref(), Some("Postal Ballot"));
    }

    #[test]
    fn parses_board_meeting_and_insider() {
        let board = parse_bse_description(
            "MEETING DATE : 08-Sep-2026 | PURPOSE : A.G.M.;General;Reduction of Capital",
        );
        assert_eq!(board.meeting_date, Some(date(2026, 9, 8)));
        assert_eq!(
            board.purpose.as_deref(),
            Some("A.G.M.;General;Reduction of Capital")
        );
        let insider = parse_bse_description("TYPE OF SECURITY : Equity Shares");
        assert_eq!(insider.type_of_security.as_deref(), Some("Equity Shares"));
    }

    #[test]
    fn infers_pub_date_from_rss_not_record_date() {
        let parsed = parse_bse_description(
            "SEGMENT : EQUITY | PURPOSE : Final Dividend | RD Date : 08 Sep, 2026",
        );
        assert_eq!(parsed.inferred_pub_date(""), None);
        assert_eq!(
            parsed.inferred_pub_date("9/8/2026 2:46:53 PM"),
            parse_bse_datetime("9/8/2026 2:46:53 PM")
        );
        let with_as_on = parse_bse_description("AS ON DATE : 08-Sep-2026");
        assert_eq!(
            with_as_on.inferred_pub_date(""),
            parse_bse_datetime("08-Sep-2026")
        );
    }
}
