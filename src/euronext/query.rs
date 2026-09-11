//! List sort/filter helpers for Euronext RSS items (no satellite tables).

use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Select};

use super::{
    entities::item::{self, Entity as ItemEntity},
    feeds::{EuronextExtraField, EuronextFeedKind},
};
use crate::list_filters::{
    FilterField, FilterKind, apply_datetime_day, apply_text_contains, filter_value,
};

pub fn sort_direction(sort: &str, key: &str) -> Option<bool> {
    let sort = sort.trim();
    let desc = format!("{key} DESC");
    let asc = format!("{key} ASC");
    if sort.eq_ignore_ascii_case(&desc) {
        Some(true)
    } else if sort.eq_ignore_ascii_case(&asc) || sort.eq_ignore_ascii_case(key) {
        Some(false)
    } else {
        None
    }
}

fn order(query: Select<item::Entity>, col: item::Column, desc: bool) -> Select<item::Entity> {
    if desc {
        query.order_by_desc(col)
    } else {
        query.order_by_asc(col)
    }
}

pub fn list_select(kind: EuronextFeedKind) -> Select<item::Entity> {
    ItemEntity::find().filter(item::Column::FeedKind.eq(kind.slug()))
}

pub fn apply_sort(
    query: Select<item::Entity>,
    kind: EuronextFeedKind,
    sort: &str,
) -> Select<item::Entity> {
    if let Some(desc) = sort_direction(sort, "Title") {
        return order(query, item::Column::Title, desc);
    }
    if let Some(desc) = sort_direction(sort, "PubDate") {
        return order(query, item::Column::PubDate, desc);
    }
    if let Some(desc) = sort_direction(sort, "Description") {
        return order(query, item::Column::Description, desc);
    }
    for field in kind.extra_list_fields() {
        if let Some(desc) = sort_direction(sort, field.sort_key()) {
            let col = match field {
                EuronextExtraField::Ticker => item::Column::CompanyTicker,
                EuronextExtraField::Company => item::Column::CompanyName,
            };
            return order(query, col, desc);
        }
    }
    query.order_by_desc(item::Column::PubDate)
}

pub fn apply_filters(
    mut query: Select<item::Entity>,
    kind: EuronextFeedKind,
    filters: &HashMap<String, String>,
) -> Select<item::Entity> {
    if let Some(v) = filter_value(filters, "Title") {
        query = apply_text_contains(query, item::Column::Title, v);
    }
    if let Some(v) = filter_value(filters, "PubDate") {
        query = apply_datetime_day(query, item::Column::PubDate, v);
    }
    if let Some(v) = filter_value(filters, "Description") {
        query = apply_text_contains(query, item::Column::Description, v);
    }
    for field in kind.extra_list_fields() {
        if let Some(v) = filter_value(filters, field.sort_key()) {
            let col = match field {
                EuronextExtraField::Ticker => item::Column::CompanyTicker,
                EuronextExtraField::Company => item::Column::CompanyName,
            };
            query = apply_text_contains(query, col, v);
        }
    }
    query
}

pub fn filter_fields(kind: EuronextFeedKind) -> Vec<FilterField> {
    let mut fields = vec![
        FilterField {
            key: "Title",
            label: "Title",
            kind: FilterKind::Text,
        },
        FilterField {
            key: "PubDate",
            label: "PubDate",
            kind: FilterKind::Date,
        },
    ];
    for field in kind.extra_list_fields() {
        fields.push(FilterField {
            key: field.sort_key(),
            label: field.label(),
            kind: FilterKind::Text,
        });
    }
    if kind.shows_description_column() {
        fields.push(FilterField {
            key: "Description",
            label: "Description",
            kind: FilterKind::Text,
        });
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::filter_fields;
    use crate::euronext::feeds::EuronextFeedKind;
    use crate::list_filters::FilterKind;

    #[test]
    fn filter_fields_include_base_columns() {
        for kind in EuronextFeedKind::ALL {
            let fields = filter_fields(*kind);
            assert_eq!(fields[0].key, "Title");
            assert_eq!(fields[1].key, "PubDate");
            assert_eq!(fields[1].kind, FilterKind::Date);
            let keys: Vec<_> = fields.iter().map(|f| f.key).collect();
            assert!(keys.contains(&"Description"));
            if kind.shows_company_columns() {
                assert!(keys.contains(&"Ticker"));
                assert!(keys.contains(&"Company"));
            } else {
                assert!(!keys.contains(&"Ticker"));
                assert!(!keys.contains(&"Company"));
            }
        }
    }
}
