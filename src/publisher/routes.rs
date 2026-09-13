use super::{
    handlers,
    keys::{
        SubscriberBulkDeleteModalKey, SubscriberBulkSendEditLinkModalKey, SubscriberDeleteModalKey,
        SubscriberSendEditLinkModalKey, SubscriberTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: PublisherTag;
    routes: [
        get SubscriberDefaultRouteTag, "/publisher/subscribers", handlers::list, fragment(SubscriberTableKey);
        get SubscriberCreateGetRouteTag, "/publisher/subscribers/create", handlers::create_get, modal;
        post SubscriberCreatePostRouteTag, "/publisher/subscribers/create", handlers::create_post;
        get SubscriberSendEmailGetRouteTag, "/publisher/subscribers/send-email", handlers::send_email_get, modal;
        post SubscriberSendEmailPostRouteTag, "/publisher/subscribers/send-email", handlers::send_email_post;
        get SubscriberBulkDeleteGetRouteTag, "/publisher/subscribers/bulk-delete", handlers::bulk_delete_get, modal;
        post SubscriberBulkDeletePostRouteTag, "/publisher/subscribers/bulk-delete", bare handlers::bulk_delete_post, fragment(SubscriberBulkDeleteModalKey);
        get SubscriberBulkSendEditLinkGetRouteTag, "/publisher/subscribers/bulk-send-edit-link", handlers::bulk_send_edit_link_get, modal;
        post SubscriberBulkSendEditLinkPostRouteTag, "/publisher/subscribers/bulk-send-edit-link", bare handlers::bulk_send_edit_link_post, fragment(SubscriberBulkSendEditLinkModalKey);
        get SubscriberDetailRouteTag, "/publisher/subscribers/{id}", handlers::detail;
        get SubscriberEditGetRouteTag, "/publisher/subscribers/{id}/edit", handlers::edit_get, modal;
        post SubscriberEditPostRouteTag, "/publisher/subscribers/{id}/edit", handlers::edit_post;
        get SubscriberSendEditLinkGetRouteTag, "/publisher/subscribers/{id}/send-edit-link", handlers::send_edit_link_get, modal;
        post SubscriberSendEditLinkPostRouteTag, "/publisher/subscribers/{id}/send-edit-link", bare handlers::send_edit_link_post, fragment(SubscriberSendEditLinkModalKey);
        get SubscriberDeleteGetRouteTag, "/publisher/subscribers/{id}/delete", handlers::delete_get, modal;
        post SubscriberDeletePostRouteTag, "/publisher/subscribers/{id}/delete", bare handlers::delete_post, fragment(SubscriberDeleteModalKey);
        get SubscribeExchangesRouteTag, "/subscribe/exchanges", bare handlers::subscribe_exchanges, raw;
        get SubscribeEventTypesRouteTag, "/subscribe/event-types", bare handlers::subscribe_event_types, raw;
        post SubscribePostRouteTag, "/subscribe/join", bare handlers::subscribe_post, redirect;
        get SubscribeEditRouteTag, "/subscribe/edit", bare handlers::subscribe_edit_get, raw;
        post SubscribeEditPostRouteTag, "/subscribe/edit", bare handlers::subscribe_edit_post, raw;
        post SubscribeCancelPostRouteTag, "/subscribe/cancel", bare handlers::subscribe_cancel_post, raw;
        get PublisherPrefsGetRouteTag, "/publisher/preferences", handlers::preferences_get;
        post PublisherPrefsPostRouteTag, "/publisher/preferences", handlers::preferences_post;
    ]
}
