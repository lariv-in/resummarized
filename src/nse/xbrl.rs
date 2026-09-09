//! First-seen text facts from an XBRL instance (namespaces ignored).

use chrono::NaiveDate;
use lariv_rs::datetime::format_date;
use quick_xml::Reader;
use quick_xml::events::Event;
use sea_orm::ActiveValue::Set;
use std::collections::HashMap;

const SKIP: &[&str] = &[
    "schemaRef",
    "context",
    "unit",
    "entity",
    "period",
    "scenario",
    "identifier",
    "explicitMember",
    "typedMember",
    "measure",
    "divide",
    "unitNumerator",
    "unitDenominator",
    "startDate",
    "endDate",
    "instant",
    "StatementOfDeviationDomain",
    "ObjectDomain",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XbrlFact {
    pub name: String,
    pub text: String,
    pub context: String,
}

/// Every non-empty fact in document order (namespaces ignored).
pub fn all_text_facts(xml: &str) -> Vec<XbrlFact> {
    let mut reader = Reader::from_str(xml.trim_start_matches('\u{feff}'));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut facts = Vec::new();
    let mut pending: Option<XbrlFact> = None;
    let mut skip_depth = 0u32;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = local_name(e.local_name().as_ref()).to_string();
                let mut context = String::new();
                for attr in e.attributes().with_checks(false).flatten() {
                    if local_name(attr.key.as_ref()) == "contextRef"
                        && let Ok(v) = attr.unescape_value()
                    {
                        context = v.into_owned();
                    }
                }
                if skip_depth > 0 || SKIP.contains(&local.as_str()) {
                    skip_depth += 1;
                    pending = None;
                } else {
                    pending = Some(XbrlFact {
                        name: local,
                        text: String::new(),
                        context,
                    });
                }
            }
            Ok(Event::Empty(_)) => {}
            Ok(Event::Text(t)) => {
                if skip_depth == 0
                    && let Some(fact) = &mut pending
                    && fact.text.is_empty()
                    && let Ok(text) = t.unescape()
                {
                    let text = text.trim();
                    if !text.is_empty() {
                        fact.text = text.to_string();
                    }
                }
            }
            Ok(Event::End(_)) => {
                skip_depth = skip_depth.saturating_sub(1);
                if let Some(fact) = pending.take()
                    && !fact.text.is_empty()
                {
                    facts.push(fact);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    facts
}

/// Local-name → first non-empty text, skipping XBRL scaffolding.
pub fn first_text_facts(xml: &str) -> HashMap<String, String> {
    let mut facts = HashMap::new();
    for fact in all_text_facts(xml) {
        facts.entry(fact.name).or_insert(fact.text);
    }
    facts
}

fn local_name(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap_or("")
}

pub(super) fn take(map: &HashMap<String, String>, key: &str) -> Option<String> {
    map.get(key)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub(super) fn take_clean(map: &HashMap<String, String>, key: &str) -> Option<String> {
    take(map, key).filter(|s| !s.bytes().all(|b| b == b'*'))
}

pub(super) fn take_yes_no(map: &HashMap<String, String>, key: &str) -> Option<String> {
    take_clean(map, key).map(|s| match s.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" => "Yes".to_string(),
        "false" | "no" | "0" => "No".to_string(),
        _ => s,
    })
}

pub(super) fn take_pct(map: &HashMap<String, String>, key: &str) -> Option<String> {
    take_clean(map, key).map(|s| fraction_to_percent(&s))
}

pub(super) fn fraction_to_percent(s: &str) -> String {
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

pub(super) fn take_date(map: &HashMap<String, String>, key: &str) -> Option<NaiveDate> {
    take(map, key).and_then(|s| crate::dates::parse_date(&s))
}

pub(super) fn set_opt(slot: &mut sea_orm::ActiveValue<Option<String>>, value: &Option<String>) {
    if let Some(v) = value {
        *slot = Set(Some(v.clone()));
    }
}

pub(super) fn set_opt_date(
    slot: &mut sea_orm::ActiveValue<Option<NaiveDate>>,
    value: Option<NaiveDate>,
) {
    if let Some(v) = value {
        *slot = Set(Some(v));
    }
}

pub(super) fn opt_str(v: &Option<String>) -> String {
    v.clone().unwrap_or_default()
}

pub(super) fn opt_date(v: Option<NaiveDate>) -> String {
    v.map(format_date).unwrap_or_default()
}
