#![allow(clippy::redundant_field_names)] // generated route tags: `{ feed: feed }`

use super::{feeds::JpxFeedKind, handlers, keys::ItemTableKey};

lariv_rs::define_plugin_routes! {
    plugin: JpxTag;
    routes: [
        get FeedListRouteTag, "/jpx/{feed}", handlers::list, fragment(ItemTableKey);
        get ItemDetailRouteTag, "/jpx/{feed}/items/{id}", handlers::detail;
        post FeedRefreshRouteTag, "/jpx/{feed}/refresh", bare handlers::refresh, redirect;
    ]
}

pub fn feed_list_url(slug: &str) -> String {
    FeedListRouteTag::new(slug.to_string()).url()
}

pub fn default_feed_url() -> String {
    feed_list_url(JpxFeedKind::ALL[0].slug())
}
