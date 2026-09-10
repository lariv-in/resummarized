use super::{
    handlers,
    keys::{SubscriberDeleteModalKey, SubscriberTableKey},
};

lariv_rs::define_plugin_routes! {
    plugin: PublisherTag;
    routes: [
        get SubscriberDefaultRouteTag, "/publisher/subscribers", handlers::list, fragment(SubscriberTableKey);
        get SubscriberCreateGetRouteTag, "/publisher/subscribers/create", handlers::create_get, modal;
        post SubscriberCreatePostRouteTag, "/publisher/subscribers/create", handlers::create_post;
        get SubscriberDetailRouteTag, "/publisher/subscribers/{id}", handlers::detail;
        get SubscriberEditGetRouteTag, "/publisher/subscribers/{id}/edit", handlers::edit_get, modal;
        post SubscriberEditPostRouteTag, "/publisher/subscribers/{id}/edit", handlers::edit_post;
        get SubscriberDeleteGetRouteTag, "/publisher/subscribers/{id}/delete", handlers::delete_get, modal;
        post SubscriberDeletePostRouteTag, "/publisher/subscribers/{id}/delete", bare handlers::delete_post, fragment(SubscriberDeleteModalKey);
    ]
}
