use super::routes::default_feed_url;

lariv_rs::define_register_apps! {
    plugin: super::NasdaqTag;
    key: "resummarized-nasdaq";
    name: "NASDAQ";
    href: default_feed_url();
    icon: "rss";
    roles: ["superuser", "admin"];
}
