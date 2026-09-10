//! Breadcrumb trails for Publisher pages.

use lariv_rs::components::{Crumb, breadcrumbs};
use maud::Markup;

use super::routes::{SubscriberDefaultRouteTag, SubscriberDetailRouteTag};

pub fn subscribers_list_crumbs() -> Markup {
    breadcrumbs(&[Crumb {
        label: "Subscribers",
        href: None,
    }])
}

pub fn subscriber_crumbs(email: &str, id: i64, action: Option<&str>) -> Markup {
    let list_url = SubscriberDefaultRouteTag.url();
    let detail_url = SubscriberDetailRouteTag::new(id).url();
    match action {
        None => breadcrumbs(&[
            Crumb {
                label: "Subscribers",
                href: Some(&list_url),
            },
            Crumb {
                label: email,
                href: None,
            },
        ]),
        Some(act) => breadcrumbs(&[
            Crumb {
                label: "Subscribers",
                href: Some(&list_url),
            },
            Crumb {
                label: email,
                href: Some(&detail_url),
            },
            Crumb {
                label: act,
                href: None,
            },
        ]),
    }
}
