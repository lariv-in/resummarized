//! Typed [`CreateModal`] wiring for Publisher swap keys.

use super::keys::SubscriberCreateModalKey;
use super::routes::{SubscriberCreateGetRouteTag, SubscriberCreatePostRouteTag};

lariv_rs::impl_create_modal!(
    SubscriberCreateModalKey,
    SubscriberCreateGetRouteTag,
    SubscriberCreatePostRouteTag,
    "publisher.SubscriberCreateForm"
);
