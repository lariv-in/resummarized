use super::routes::default_feed_url;

lariv_rs::define_register_apps! {
    plugin: super::NseTag;
    key: "resummarized-nse";
    name: "NSE";
    href: default_feed_url();
    icon: "rss";
    roles: ["superuser", "admin"];
}
