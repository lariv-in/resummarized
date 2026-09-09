use frunk::Generic;
use lariv_rs::{
    capability::define_register_items,
    components::{
        Crumb, DetailHeader, FieldText, LayoutMain, LayoutSidebar, ObjectList, PaginationPage,
        ShellChrome, ShellScaffold, SidebarMenu, SidebarNavLink, SlotCapability, SlotRegistrar,
        SwapKey, TableColumnHeader, TablePagination, TableRow, breadcrumbs, button_post_route,
        column_sort_url, container_column, data_table_list_refresh, detail, detail_header,
        field_text, label, layout_main, layout_sidebar, pagination_pages, row_attr_navigate_route,
        shell_scaffold, sidebar_menu, sidebar_nav_items_pane, sort_indicator, table_pagination,
    },
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
};
use maud::{Markup, html};

use super::feeds::BseFeedKind;
use super::keys::ItemTableKey;
use super::routes::{FeedRefreshRouteTag, ItemDetailRouteTag, feed_list_url};

define_register_items! {
    plugin: BseTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        ItemListIdx: FeedItemListPageTag => FeedItemListPage,
        ItemDetailIdx: FeedItemDetailPageTag => FeedItemDetailPage,
    ]
}

define_register_items! {
    plugin: BseTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

fn app_scaffold(
    title: &str,
    chrome: &ShellChrome,
    sidebar: Markup,
    crumbs: Markup,
    body: Markup,
) -> Markup {
    shell_scaffold(ShellScaffold {
        title,
        registry_head: chrome.head.clone(),
        topbar_items: chrome.topbar_items.clone(),
        right_sidebar: chrome.right_sidebar.clone(),
        sidebar,
        breadcrumbs: crumbs,
        body,
        ..Default::default()
    })
}

fn scaffold_pane(
    sidebar: Markup,
    crumbs: Markup,
    body: Markup,
) -> lariv_rs::components::AppLayoutHtml {
    layout_sidebar(LayoutSidebar {
        sidebar,
        breadcrumbs: crumbs,
        content: body,
    })
}

fn scaffold_main(crumbs: Markup, body: Markup) -> lariv_rs::components::MainContentHtml {
    layout_main(LayoutMain {
        breadcrumbs: crumbs,
        content: body,
    })
}

fn bse_menu(current_path: &str) -> Markup {
    let feed_meta: Vec<(&'static str, &'static str, String)> = BseFeedKind::ALL
        .iter()
        .map(|kind| (kind.slug(), kind.display_name(), feed_list_url(kind.slug())))
        .collect();
    let mut links: Vec<SidebarNavLink<'_>> = Vec::with_capacity(feed_meta.len());
    for (slug, name, url) in &feed_meta {
        links.push(SidebarNavLink {
            key: slug,
            title: name,
            url: url.as_str(),
            icon_name: None,
            match_prefixes: &[],
        });
    }
    sidebar_menu(SidebarMenu {
        title: "BSE",
        children: sidebar_nav_items_pane(&links, current_path),
    })
}

fn feed_list_crumbs(feed_name: &str) -> Markup {
    breadcrumbs(&[
        Crumb {
            label: "BSE",
            href: None,
        },
        Crumb {
            label: feed_name,
            href: None,
        },
    ])
}

fn item_detail_crumbs(feed_slug: &str, feed_name: &str, title: &str) -> Markup {
    let list_url = feed_list_url(feed_slug);
    breadcrumbs(&[
        Crumb {
            label: "BSE",
            href: None,
        },
        Crumb {
            label: feed_name,
            href: Some(&list_url),
        },
        Crumb {
            label: title,
            href: None,
        },
    ])
}

fn render_pagination<K: SwapKey>(
    path_and_query: &str,
    number: u32,
    num_pages: u32,
    push_url: bool,
) -> Markup {
    let owned = pagination_pages(path_and_query, number, num_pages, push_url);
    let pages: Vec<PaginationPage<'_>> = owned
        .iter()
        .map(|(ellipsis, url, push_url, active, label)| PaginationPage {
            ellipsis: *ellipsis,
            url: url.as_str(),
            push_url: *push_url,
            active: *active,
            label: label.as_str(),
        })
        .collect();
    table_pagination(TablePagination {
        pages: &pages,
        hx_target: K::SELECTOR,
    })
}

#[derive(Clone, Copy)]
pub struct FeedListColumn {
    pub sort_key: &'static str,
    pub label: &'static str,
}

#[derive(Clone)]
pub struct FeedItemRow {
    pub id: i64,
    pub title: String,
    pub pub_date: String,
    pub extra: Vec<String>,
    pub description: String,
}

#[derive(Generic)]
pub struct FeedItemListPage {
    pub feed_slug: String,
    pub feed_name: String,
    pub extra_fields: Vec<FeedListColumn>,
    pub show_description: bool,
    pub items: ObjectList<FeedItemRow>,
    pub sort: String,
    pub path_and_query: String,
}

impl FeedItemListPage {
    pub fn render_table(&self) -> Markup {
        let title_sort = column_sort_url(&self.path_and_query, "Title", &self.sort);
        let pub_sort = column_sort_url(&self.path_and_query, "PubDate", &self.sort);
        let extra_sorts: Vec<String> = self
            .extra_fields
            .iter()
            .map(|f| column_sort_url(&self.path_and_query, f.sort_key, &self.sort))
            .collect();
        let title_label = format!("Title{}", sort_indicator(&self.sort, "Title"));
        let pub_label = format!("PubDate{}", sort_indicator(&self.sort, "PubDate"));
        let extra_labels: Vec<String> = self
            .extra_fields
            .iter()
            .map(|f| format!("{}{}", f.label, sort_indicator(&self.sort, f.sort_key)))
            .collect();
        let mut headers = vec![
            TableColumnHeader {
                key: "Title",
                label: &title_label,
                sort_url: Some(&title_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "PubDate",
                label: &pub_label,
                sort_url: Some(&pub_sort),
                push_url: true,
            },
        ];
        for (i, field) in self.extra_fields.iter().enumerate() {
            headers.push(TableColumnHeader {
                key: field.sort_key,
                label: &extra_labels[i],
                sort_url: Some(&extra_sorts[i]),
                push_url: true,
            });
        }
        if self.show_description {
            headers.push(TableColumnHeader {
                key: "Description",
                label: "Description",
                sort_url: None,
                push_url: true,
            });
        }
        let rows: Vec<TableRow> = self
            .items
            .items
            .iter()
            .map(|item| {
                let mut cells = vec![
                    field_text(FieldText {
                        value: &item.title,
                        classes: "",
                    }),
                    field_text(FieldText {
                        value: &item.pub_date,
                        classes: "",
                    }),
                ];
                for value in &item.extra {
                    cells.push(field_text(FieldText { value, classes: "" }));
                }
                if self.show_description {
                    cells.push(field_text(FieldText {
                        value: &item.description,
                        classes: "",
                    }));
                }
                TableRow {
                    attrs: row_attr_navigate_route(ItemDetailRouteTag::new(
                        self.feed_slug.clone(),
                        item.id,
                    )),
                    cells,
                }
            })
            .collect();
        let actions = html! {
            (button_post_route(
                FeedRefreshRouteTag::new(self.feed_slug.clone()),
                "Refresh",
                "btn-outline btn-sm",
            ))
        };
        let pagination = render_pagination::<ItemTableKey>(
            &self.path_and_query,
            self.items.number,
            self.items.num_pages,
            true,
        );
        data_table_list_refresh::<ItemTableKey>(
            "",
            actions,
            &headers,
            &rows,
            pagination,
            &self.path_and_query,
        )
    }
}

impl RenderAppPane for FeedItemListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            bse_menu(&self.path_and_query),
            feed_list_crumbs(&self.feed_name),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(feed_list_crumbs(&self.feed_name), self.render_table())
    }
}

impl RenderTemplate for FeedItemListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — BSE", self.feed_name),
            chrome,
            bse_menu(&self.path_and_query),
            feed_list_crumbs(&self.feed_name),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct FeedItemDetailPage {
    pub feed_slug: String,
    pub feed_name: String,
    pub title: String,
    pub link: Option<String>,
    pub description: String,
    pub pub_date: String,
    pub extra_fields: Vec<(String, String)>,
}

impl FeedItemDetailPage {
    fn pane_body(&self) -> Markup {
        detail(html! {
            (container_column(
                "",
                html! {
                    (detail_header(DetailHeader {
                        title: &self.title,
                        actions: html! {},
                    }))
                    (label("Feed", field_text(FieldText {
                        value: &self.feed_name,
                        classes: "",
                    })))
                    (label("PubDate", field_text(FieldText {
                        value: &self.pub_date,
                        classes: "",
                    })))
                    @for (name, value) in &self.extra_fields {
                        (label(name, field_text(FieldText {
                            value,
                            classes: "",
                        })))
                    }
                    @if let Some(link) = &self.link {
                        (label("Link", html! {
                            a href=(link) class="link break-all" target="_blank" rel="noopener noreferrer" { (link) }
                        }))
                    }
                    @if !self.description.is_empty() {
                        (label("Description", field_text(FieldText {
                            value: &self.description,
                            classes: "whitespace-pre-wrap",
                        })))
                    }
                },
            ))
        })
    }
}

impl RenderAppPane for FeedItemDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            bse_menu(&feed_list_url(&self.feed_slug)),
            item_detail_crumbs(&self.feed_slug, &self.feed_name, &self.title),
            self.pane_body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(
            item_detail_crumbs(&self.feed_slug, &self.feed_name, &self.title),
            self.pane_body(),
        )
    }
}

impl RenderTemplate for FeedItemDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            &format!("{} — BSE", self.title),
            chrome,
            bse_menu(&feed_list_url(&self.feed_slug)),
            item_detail_crumbs(&self.feed_slug, &self.feed_name, &self.title),
            self.pane_body(),
        )
    }
}
