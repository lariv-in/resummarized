use super::routes::default_feed_url;

lariv_rs::define_register_apps! {
    plugin: super::EuronextTag;
    key: "resummarized-euronext";
    name: "Euronext";
    href: default_feed_url();
    icon: "rss";
    roles: ["superuser", "admin"];
}
