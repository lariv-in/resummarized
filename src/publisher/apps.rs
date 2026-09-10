use super::routes::SubscriberDefaultRouteTag;

lariv_rs::define_register_apps! {
    plugin: super::PublisherTag;
    key: "resummarized-publisher";
    name: "Publisher";
    href: SubscriberDefaultRouteTag.url();
    icon: "newspaper";
    roles: ["superuser"];
}
