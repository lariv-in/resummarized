//! Shared list-table filter query helpers and filter-form inputs for NSE/BSE/Nasdaq.

use std::collections::HashMap;

use lariv_rs::components::{InputDate, InputText, input_date, input_text};
use maud::{Markup, html};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select, sea_query::Expr};

/// Widget used in the table filter dropdown.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FilterKind {
    Text,
    Date,
}

/// Stored column type. [`DateTime`] uses a date widget and an IST-day range.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueKind {
    Text,
    Date,
    DateTime,
}

impl ValueKind {
    pub fn filter_kind(self) -> FilterKind {
        match self {
            Self::Text => FilterKind::Text,
            Self::Date | Self::DateTime => FilterKind::Date,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FilterField {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: FilterKind,
}

pub const FILTER_PANEL_CLASSES: &str = "card w-80 max-h-[32rem] overflow-y-auto my-1.5 card-body shadow dropdown-content border border-base-300 rounded-box z-2 bg-base-100";

pub fn nonempty_filters(filters: &HashMap<String, String>) -> impl Iterator<Item = (&str, &str)> {
    filters.iter().filter_map(|(k, v)| {
        let v = v.trim();
        if v.is_empty() {
            None
        } else {
            Some((k.as_str(), v))
        }
    })
}

pub fn filter_value<'a>(filters: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    filters.get(key).map(|s| s.trim()).filter(|s| !s.is_empty())
}

fn never<E: EntityTrait>(query: Select<E>) -> Select<E> {
    query.filter(Expr::cust("0 = 1"))
}

pub fn apply_text_contains<E: EntityTrait, C: ColumnTrait>(
    query: Select<E>,
    col: C,
    value: &str,
) -> Select<E> {
    query.filter(col.contains(value))
}

pub fn apply_date_eq<E: EntityTrait, C: ColumnTrait>(
    query: Select<E>,
    col: C,
    value: &str,
) -> Select<E> {
    match crate::dates::parse_date(value) {
        Some(d) => query.filter(col.eq(d)),
        None => never(query),
    }
}

pub fn apply_datetime_day<E: EntityTrait, C: ColumnTrait>(
    query: Select<E>,
    col: C,
    value: &str,
) -> Select<E> {
    let Some(d) = crate::dates::parse_date(value) else {
        return never(query);
    };
    match (
        crate::dates::date_start_ist(d),
        crate::dates::date_end_ist(d),
    ) {
        (Some(start), Some(end)) => query.filter(col.gte(start)).filter(col.lte(end)),
        _ => never(query),
    }
}

pub fn apply_value_filter<E: EntityTrait, C: ColumnTrait>(
    query: Select<E>,
    col: C,
    kind: ValueKind,
    value: &str,
) -> Select<E> {
    match kind {
        ValueKind::Text => apply_text_contains(query, col, value),
        ValueKind::Date => apply_date_eq(query, col, value),
        ValueKind::DateTime => apply_datetime_day(query, col, value),
    }
}

pub fn render_filter_inputs(fields: &[FilterField], values: &HashMap<String, String>) -> Markup {
    html! {
        @for field in fields {
            @let value = values.get(field.key).map(String::as_str).unwrap_or("");
            @match field.kind {
                FilterKind::Text => {
                    (input_text(InputText {
                        label: field.label,
                        name: field.key,
                        value,
                        ..Default::default()
                    }))
                }
                FilterKind::Date => {
                    (input_date(InputDate {
                        label: field.label,
                        name: field.key,
                        value,
                        ..Default::default()
                    }))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_blank_filter_values() {
        let mut filters = HashMap::new();
        filters.insert("Title".into(), "  ".into());
        filters.insert("Subject".into(), "board".into());
        let got: Vec<_> = nonempty_filters(&filters).collect();
        assert_eq!(got, vec![("Subject", "board")]);
        assert!(filter_value(&filters, "Title").is_none());
        assert_eq!(filter_value(&filters, "Subject"), Some("board"));
    }

    #[test]
    fn datetime_kind_uses_date_widget() {
        assert_eq!(ValueKind::DateTime.filter_kind(), FilterKind::Date);
        assert_eq!(ValueKind::Date.filter_kind(), FilterKind::Date);
        assert_eq!(ValueKind::Text.filter_kind(), FilterKind::Text);
    }
}
