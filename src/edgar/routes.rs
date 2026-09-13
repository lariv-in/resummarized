#![allow(clippy::redundant_field_names)]

use super::{feeds::EdgarFeedKind, handlers, keys::ItemTableKey};

lariv_rs::define_plugin_routes! {
    plugin: EdgarTag;
    routes: [
        get FeedListRouteTag, "/edgar/{feed}", handlers::list, fragment(ItemTableKey);
        get ItemDetailRouteTag, "/edgar/{feed}/items/{id}", handlers::detail;
        post FeedRefreshRouteTag, "/edgar/{feed}/refresh", bare handlers::refresh, redirect;
    ]
}

pub fn feed_list_url(slug: &str) -> String {
    FeedListRouteTag::new(slug.to_string()).url()
}

pub fn default_feed_url() -> String {
    feed_list_url(EdgarFeedKind::ALL[0].slug())
}
