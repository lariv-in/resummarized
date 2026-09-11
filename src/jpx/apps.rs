use super::routes::default_feed_url;

lariv_rs::define_register_apps! {
    plugin: super::JpxTag;
    key: "resummarized-jpx";
    name: "JPX";
    href: default_feed_url();
    icon: "rss";
    roles: ["superuser", "admin"];
}
