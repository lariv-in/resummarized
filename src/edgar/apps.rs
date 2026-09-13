use super::routes::default_feed_url;

lariv_rs::define_register_apps! {
    plugin: super::EdgarTag;
    key: "resummarized-edgar";
    name: "SEC EDGAR";
    href: default_feed_url();
    icon: "building-library";
    roles: ["superuser", "admin"];
}
