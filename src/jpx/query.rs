//! List sort/filter helpers for JPX RSS items (no satellite tables).

use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Select};

use super::{
    entities::item::{self, Entity as ItemEntity},
    feeds::JpxFeedKind,
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

pub fn list_select(kind: JpxFeedKind) -> Select<item::Entity> {
    ItemEntity::find().filter(item::Column::FeedKind.eq(kind.slug()))
}

pub fn apply_sort(query: Select<item::Entity>, sort: &str) -> Select<item::Entity> {
    if let Some(desc) = sort_direction(sort, "Title") {
        return order(query, item::Column::Title, desc);
    }
    if let Some(desc) = sort_direction(sort, "PubDate") {
        return order(query, item::Column::PubDate, desc);
    }
    query.order_by_desc(item::Column::PubDate)
}

pub fn apply_filters(
    mut query: Select<item::Entity>,
    filters: &HashMap<String, String>,
) -> Select<item::Entity> {
    if let Some(v) = filter_value(filters, "Title") {
        query = apply_text_contains(query, item::Column::Title, v);
    }
    if let Some(v) = filter_value(filters, "PubDate") {
        query = apply_datetime_day(query, item::Column::PubDate, v);
    }
    query
}

pub fn filter_fields(_kind: JpxFeedKind) -> Vec<FilterField> {
    vec![
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
    ]
}

#[cfg(test)]
mod tests {
    use super::filter_fields;
    use crate::jpx::feeds::JpxFeedKind;
    use crate::list_filters::FilterKind;

    #[test]
    fn filter_fields_include_base_columns_only() {
        for kind in JpxFeedKind::ALL {
            let fields = filter_fields(*kind);
            assert_eq!(fields[0].key, "Title");
            assert_eq!(fields[1].key, "PubDate");
            assert_eq!(fields[1].kind, FilterKind::Date);
            let keys: Vec<_> = fields.iter().map(|f| f.key).collect();
            assert!(!keys.contains(&"Description"));
            assert_eq!(fields.len(), 2);
        }
    }
}
