use super::routes::default_feed_url;

lariv_rs::define_register_apps! {
    plugin: super::BseTag;
    key: "resummarized-bse";
    name: "BSE";
    href: default_feed_url();
    icon: "rss";
    roles: ["superuser", "admin"];
}
