//! Shared calendar-date and datetime parsing for this deployment.
//!
//! Every datetime wall clock is interpreted as IST (`Asia/Kolkata`) and stored
//! as UTC. RFC 2822 / RFC 3339 offsets (including `GMT`) are ignored; the numbers
//! in the string are IST, not the labeled zone. Calendar dates have no timezone.

use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use lariv_rs::datetime::DEFAULT_TIMEZONE;

pub fn is_blank(s: &str) -> bool {
    let s = s.trim();
    s.is_empty() || s == "-"
}

fn parse_month(s: &str) -> Option<u32> {
    match s.to_ascii_uppercase().as_str() {
        "JAN" | "JANUARY" => Some(1),
        "FEB" | "FEBRUARY" => Some(2),
        "MAR" | "MARCH" => Some(3),
        "APR" | "APRIL" => Some(4),
        "MAY" => Some(5),
        "JUN" | "JUNE" => Some(6),
        "JUL" | "JULY" => Some(7),
        "AUG" | "AUGUST" => Some(8),
        "SEP" | "SEPT" | "SEPTEMBER" => Some(9),
        "OCT" | "OCTOBER" => Some(10),
        "NOV" | "NOVEMBER" => Some(11),
        "DEC" | "DECEMBER" => Some(12),
        _ => None,
    }
}

fn parse_year(s: &str) -> Option<i32> {
    match s.len() {
        2 => {
            let y: i32 = s.parse().ok()?;
            Some(if y >= 70 { 1900 + y } else { 2000 + y })
        }
        4 => s.parse().ok(),
        _ => None,
    }
}

fn parse_dashed_mon(s: &str) -> Option<NaiveDate> {
    let mut parts = s.split('-');
    let day: u32 = parts.next()?.parse().ok()?;
    let month = parse_month(parts.next()?)?;
    let year = parse_year(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    NaiveDate::from_ymd_opt(year, month, day)
}

fn parse_numeric_dmy(s: &str) -> Option<NaiveDate> {
    let mut parts = s.split('-');
    let a = parts.next()?;
    let b = parts.next()?;
    let c = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    if a.len() == 4 {
        let year: i32 = a.parse().ok()?;
        let month: u32 = b.parse().ok()?;
        let day: u32 = c.parse().ok()?;
        return NaiveDate::from_ymd_opt(year, month, day);
    }
    let day: u32 = a.parse().ok()?;
    let month: u32 = b.parse().ok()?;
    let year = parse_year(c)?;
    NaiveDate::from_ymd_opt(year, month, day)
}

/// Slash dates from BSE SENSEX are US `M/D/YYYY` (`9/8/2026` = 8 Sep).
fn parse_slash_mdy(s: &str) -> Option<NaiveDate> {
    let mut parts = s.split('/');
    let a: u32 = parts.next()?.parse().ok()?;
    let b: u32 = parts.next()?.parse().ok()?;
    let year = parse_year(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    let (month, day) = if a > 12 { (b, a) } else { (a, b) };
    NaiveDate::from_ymd_opt(year, month, day)
}

fn parse_d_mon_y(s: &str) -> Option<NaiveDate> {
    let s = s.replace(',', " ");
    let mut parts = s.split_whitespace();
    let day: u32 = parts.next()?.parse().ok()?;
    let month = parse_month(parts.next()?)?;
    let year = parse_year(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    NaiveDate::from_ymd_opt(year, month, day)
}

fn parse_mon_d_y(s: &str) -> Option<NaiveDate> {
    let s = s.replace(',', " ");
    let mut parts = s.split_whitespace();
    let month = parse_month(parts.next()?)?;
    let day: u32 = parts.next()?.parse().ok()?;
    let year = parse_year(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    NaiveDate::from_ymd_opt(year, month, day)
}

/// Parse a calendar date (no timezone).
pub fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if is_blank(s) {
        return None;
    }
    parse_dashed_mon(s)
        .or_else(|| parse_numeric_dmy(s))
        .or_else(|| parse_slash_mdy(s))
        .or_else(|| parse_d_mon_y(s))
        .or_else(|| parse_mon_d_y(s))
        .or_else(|| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
}

fn frac_to_nano(s: &str) -> u32 {
    let mut padded = s.to_string();
    while padded.len() < 9 {
        padded.push('0');
    }
    padded.truncate(9);
    padded.parse().unwrap_or(0)
}

fn parse_hms(part: &str, sep: char) -> Option<(u32, u32, u32, u32)> {
    let mut bits = part.split(sep);
    let hour: u32 = bits.next()?.parse().ok()?;
    let min: u32 = bits.next()?.parse().ok()?;
    let Some(sec_part) = bits.next() else {
        return Some((hour, min, 0, 0));
    };
    let (sec, nano) = if let Some((s, f)) = sec_part.split_once('.') {
        (s.parse().ok()?, frac_to_nano(f))
    } else if let Some(frac) = bits.next() {
        (sec_part.parse().ok()?, frac_to_nano(frac))
    } else {
        (sec_part.parse().ok()?, 0)
    };
    if bits.next().is_some() {
        return None;
    }
    Some((hour, min, sec, nano))
}

fn apply_ampm(hour: u32, pm: Option<bool>) -> u32 {
    match pm {
        Some(true) if hour < 12 => hour + 12,
        Some(false) if hour == 12 => 0,
        _ => hour,
    }
}

fn parse_time_after_date(date: NaiveDate, rest: &str) -> Option<NaiveDateTime> {
    let rest = rest.trim();
    let upper = rest.to_ascii_uppercase();
    let (time_part, pm) = if let Some(t) = upper.strip_suffix("PM") {
        (t.trim(), Some(true))
    } else if let Some(t) = upper.strip_suffix("AM") {
        (t.trim(), Some(false))
    } else {
        (upper.as_str(), None)
    };
    let (hour, min, sec, nano) = parse_clock(time_part)?;
    date.and_hms_nano_opt(apply_ampm(hour, pm), min, sec, nano)
}

/// `HH:MM[:SS[.frac]]`, `HH.MM.SS[.frac]`, or mixed `HH:MM.SS` (NSE related-party).
fn parse_clock(time_part: &str) -> Option<(u32, u32, u32, u32)> {
    if time_part.contains(':') {
        let colons = time_part.bytes().filter(|&b| b == b':').count();
        if colons == 1 {
            parse_hms(&time_part.replacen('.', ":", 1), ':')
        } else {
            parse_hms(time_part, ':')
        }
    } else {
        parse_hms(time_part, '.')
    }
}

fn is_weekday(s: &str) -> bool {
    matches!(
        s.to_ascii_uppercase().as_str(),
        "MON"
            | "MONDAY"
            | "TUE"
            | "TUES"
            | "TUESDAY"
            | "WED"
            | "WEDNESDAY"
            | "THU"
            | "THUR"
            | "THURS"
            | "THURSDAY"
            | "FRI"
            | "FRIDAY"
            | "SAT"
            | "SATURDAY"
            | "SUN"
            | "SUNDAY"
    )
}

fn is_numeric_offset(s: &str) -> bool {
    let rest = s
        .strip_prefix('+')
        .or_else(|| s.strip_prefix('-'))
        .unwrap_or("");
    match rest.len() {
        2 | 4 => rest.bytes().all(|b| b.is_ascii_digit()),
        5 => {
            rest.as_bytes().get(2) == Some(&b':')
                && rest.bytes().filter(|b| b.is_ascii_digit()).count() == 4
        }
        _ => false,
    }
}

fn is_tz_token(s: &str) -> bool {
    matches!(
        s.to_ascii_uppercase().as_str(),
        "IST"
            | "GMT"
            | "UTC"
            | "UT"
            | "Z"
            | "EST"
            | "EDT"
            | "CST"
            | "CDT"
            | "MST"
            | "MDT"
            | "PST"
            | "PDT"
    ) || is_numeric_offset(s)
}

/// Drop weekday names and labeled zones. The remaining numbers are IST.
fn normalize_datetime_text(s: &str) -> String {
    let mut s = s.trim().to_string();
    if let Some((rest, last)) = s.rsplit_once(char::is_whitespace) {
        if is_tz_token(last) {
            s = rest.trim().to_string();
        }
    }
    if let Some(i) = s.rfind(['+', '-']) {
        if i > 0 && s.as_bytes()[i - 1].is_ascii_digit() && is_numeric_offset(&s[i..]) {
            s.truncate(i);
            s = s.trim().to_string();
        }
    }
    if (s.ends_with('Z') || s.ends_with('z'))
        && s.len() > 1
        && s.as_bytes()[s.len() - 2].is_ascii_digit()
    {
        s.pop();
        s = s.trim().to_string();
    }
    let stripped = s.trim();
    let stripped = if let Some((head, rest)) = stripped.split_once(',') {
        if is_weekday(head.trim()) {
            rest.trim()
        } else {
            stripped
        }
    } else {
        stripped
    };
    let stripped = if let Some((head, rest)) = stripped.split_once(char::is_whitespace) {
        if is_weekday(head) {
            rest.trim()
        } else {
            stripped
        }
    } else {
        stripped
    };
    replace_iso_t_separator(stripped)
}

fn replace_iso_t_separator(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    for (i, ch) in s.char_indices() {
        if (ch == 'T' || ch == 't')
            && i > 0
            && i + 1 < bytes.len()
            && bytes[i - 1].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
        {
            out.push(' ');
        } else {
            out.push(ch);
        }
    }
    out
}

/// Attach IST to a naive wall clock and return UTC.
pub fn ist_to_utc(naive: NaiveDateTime) -> Option<DateTime<Utc>> {
    lariv_rs::datetime::parse_timezone(DEFAULT_TIMEZONE)
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Midnight IST for a calendar date, stored as UTC.
pub fn date_start_ist(date: NaiveDate) -> Option<DateTime<Utc>> {
    ist_to_utc(date.and_hms_nano_opt(0, 0, 0, 0)?)
}

/// Last nanosecond of an IST calendar day, stored as UTC.
pub fn date_end_ist(date: NaiveDate) -> Option<DateTime<Utc>> {
    ist_to_utc(date.and_hms_nano_opt(23, 59, 59, 999_999_999)?)
}

/// Parse a search bound. RFC 3339 offsets are honored. Date-only strings use
/// IST midnight (`from`) or end of the IST day (`to`). Other datetimes follow
/// [`parse_datetime`] (IST wall clock).
pub fn parse_search_bound(s: &str, end_of_day_if_date: bool) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if is_blank(s) {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Some(date) = parse_date(s) {
        return if end_of_day_if_date {
            date_end_ist(date)
        } else {
            date_start_ist(date)
        };
    }
    parse_datetime(s)
}

/// Parse a datetime. The wall-clock numbers are always IST, even when the
/// string is labeled GMT / Z / +00:00.
pub fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if is_blank(s) {
        return None;
    }
    let s = normalize_datetime_text(s);
    if let Some(date) = parse_date(&s) {
        return date_start_ist(date);
    }
    let tokens: Vec<&str> = s.split_whitespace().collect();
    for n in 1..=tokens.len().min(4) {
        let date_str = tokens[..n].join(" ");
        let Some(date) = parse_date(&date_str) else {
            continue;
        };
        let rest = tokens[n..].join(" ");
        if rest.is_empty() {
            return date_start_ist(date);
        }
        return ist_to_utc(parse_time_after_date(date, &rest)?);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{date_end_ist, parse_date, parse_datetime, parse_search_bound};
    use chrono::{NaiveDate, TimeZone, Utc};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn rfc_and_iso_wall_clocks_are_ist_not_labeled_zone() {
        // +0530 is already IST; same instant either way.
        let ist_off = parse_datetime("Mon, 7 Sep 2026 00:00:00 +0530").unwrap();
        assert_eq!(
            ist_off,
            Utc.with_ymd_and_hms(2026, 9, 6, 18, 30, 0).unwrap()
        );
        // Labeled GMT: treat 07:32:03 as IST → 02:02 UTC.
        let gmt = parse_datetime("Tue, 08 Sep 2026 07:32:03 GMT").unwrap();
        assert_eq!(gmt, Utc.with_ymd_and_hms(2026, 9, 8, 2, 2, 3).unwrap());
        // Labeled UTC: treat 17:58:05 as IST → 12:28 UTC.
        let iso = parse_datetime("2026-09-07T17:58:05+00:00").unwrap();
        assert_eq!(iso, Utc.with_ymd_and_hms(2026, 9, 7, 12, 28, 5).unwrap());
    }

    #[test]
    fn naive_exchange_timestamps_are_ist() {
        let nse = parse_datetime("07-Sep-2026 23:28:05").unwrap();
        assert_eq!(nse, Utc.with_ymd_and_hms(2026, 9, 7, 17, 58, 5).unwrap());
        let bse = parse_datetime("9/8/2026 2:46:53 PM").unwrap();
        assert_eq!(bse, Utc.with_ymd_and_hms(2026, 9, 8, 9, 16, 53).unwrap());
        assert_eq!(parse_date("07-SEP-26"), Some(date(2026, 9, 7)));
        assert_eq!(parse_date("2026-09-07"), Some(date(2026, 9, 7)));
    }

    #[test]
    fn live_feed_formats_that_were_dropped() {
        let iso_space = parse_datetime("2026-09-07 18:52:27").unwrap();
        assert_eq!(
            iso_space,
            Utc.with_ymd_and_hms(2026, 9, 7, 13, 22, 27).unwrap()
        );
        let no_secs = parse_datetime("01-Sep-2026 11:46").unwrap();
        assert_eq!(no_secs, Utc.with_ymd_and_hms(2026, 9, 1, 6, 16, 0).unwrap());
        let mixed = parse_datetime("12-MAY-2025 22:15.59").unwrap();
        assert_eq!(
            mixed,
            Utc.with_ymd_and_hms(2025, 5, 12, 16, 45, 59).unwrap()
        );
        let date_only = parse_datetime("02-Sep-2026").unwrap();
        assert_eq!(
            date_only,
            Utc.with_ymd_and_hms(2026, 9, 1, 18, 30, 0).unwrap()
        );
        let oct = parse_datetime("31-Oct-2026 12:00:00").unwrap();
        assert_eq!(oct, Utc.with_ymd_and_hms(2026, 10, 31, 6, 30, 0).unwrap());
        assert!(parse_datetime("").is_none());
        assert!(parse_datetime("-").is_none());
    }

    #[test]
    fn date_end_ist_is_last_nanosecond_of_ist_day() {
        let end = date_end_ist(date(2026, 9, 7)).unwrap();
        assert_eq!(
            end,
            Utc.with_ymd_and_hms(2026, 9, 7, 18, 29, 59).unwrap()
                + chrono::Duration::nanoseconds(999_999_999)
        );
    }

    #[test]
    fn search_bound_honors_rfc3339_and_date_only_to() {
        let from = parse_search_bound("2026-09-07T00:00:00Z", false).unwrap();
        assert_eq!(from, Utc.with_ymd_and_hms(2026, 9, 7, 0, 0, 0).unwrap());
        let to = parse_search_bound("2026-09-07", true).unwrap();
        assert_eq!(to, date_end_ist(date(2026, 9, 7)).unwrap());
        let from_date = parse_search_bound("2026-09-07", false).unwrap();
        assert_eq!(
            from_date,
            Utc.with_ymd_and_hms(2026, 9, 6, 18, 30, 0).unwrap()
        );
        let ist_clock = parse_search_bound("07-Sep-2026 23:28:05", false).unwrap();
        assert_eq!(
            ist_clock,
            Utc.with_ymd_and_hms(2026, 9, 7, 17, 58, 5).unwrap()
        );
        assert!(parse_search_bound("", false).is_none());
        assert!(parse_search_bound("not-a-date", true).is_none());
    }
}
