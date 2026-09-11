#![allow(clippy::redundant_field_names)] // generated route tags: `{ feed: feed }`

use super::{feeds::NasdaqFeedKind, handlers, keys::ItemTableKey};

lariv_rs::define_plugin_routes! {
    plugin: NasdaqTag;
    routes: [
        get FeedListRouteTag, "/nasdaq/{feed}", handlers::list, fragment(ItemTableKey);
        get ItemDetailRouteTag, "/nasdaq/{feed}/items/{id}", handlers::detail;
        post FeedRefreshRouteTag, "/nasdaq/{feed}/refresh", bare handlers::refresh, redirect;
    ]
}

pub fn feed_list_url(slug: &str) -> String {
    FeedListRouteTag::new(slug.to_string()).url()
}

pub fn default_feed_url() -> String {
    feed_list_url(NasdaqFeedKind::ALL[0].slug())
}
