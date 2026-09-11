#![allow(clippy::redundant_field_names)] // generated route tags: `{ feed: feed }`

use super::{feeds::EuronextFeedKind, handlers, keys::ItemTableKey};

lariv_rs::define_plugin_routes! {
    plugin: EuronextTag;
    routes: [
        get FeedListRouteTag, "/euronext/{feed}", handlers::list, fragment(ItemTableKey);
        get ItemDetailRouteTag, "/euronext/{feed}/items/{id}", handlers::detail;
        post FeedRefreshRouteTag, "/euronext/{feed}/refresh", bare handlers::refresh, redirect;
    ]
}

pub fn feed_list_url(slug: &str) -> String {
    FeedListRouteTag::new(slug.to_string()).url()
}

pub fn default_feed_url() -> String {
    feed_list_url(EuronextFeedKind::ALL[0].slug())
}
