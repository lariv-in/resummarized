use frunk::Generic;
use lariv_rs::{
    components::{
        ButtonModalForm, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldDatetime, FieldText,
        FormOpts, LayoutMain, LayoutSidebar, ObjectList, PaginationPage, ShellChrome,
        ShellScaffold, SidebarMenu, SidebarMenuItem, SlotCapability, SlotRegistrar, SwapKey,
        TableButtonFilter, TableColumnHeader, TablePagination, TableRow, button_modal_form,
        button_submit, column_sort_url, container_column, data_table_list_refresh,
        delete_confirmation, detail, detail_header, field_datetime, field_text, form,
        form_hx_get_route, form_hx_post_selector, form_hx_post_url, label, layout_main,
        layout_sidebar, modal, modal_keyed, pagination_pages, row_attr_navigate_route,
        shell_scaffold, sidebar_menu, sidebar_menu_item_pane, sort_indicator, table_button_filter,
        table_create_button, table_pagination,
    },
    html_form::{FormCtx, HtmlForm},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_edit_post_url},
};
use maud::{Markup, html};

use super::crumbs::{subscriber_crumbs, subscribers_list_crumbs};
use super::forms::{
    SubscriberFilterForm, SubscriberFilterFormField, SubscriberForm, SubscriberFormField,
};
use super::keys::{
    SubscriberCreateModalKey, SubscriberDeleteModalKey, SubscriberEditModalKey, SubscriberTableKey,
};
use super::routes::{
    SubscriberCreatePostRouteTag, SubscriberDefaultRouteTag, SubscriberDeleteGetRouteTag,
    SubscriberDetailRouteTag, SubscriberEditGetRouteTag, SubscriberEditPostRouteTag,
};

const CREATE_FORM: &str = "publisher.SubscriberCreateForm";
const EDIT_FORM: &str = "publisher.SubscriberEditForm";
const DELETE_FORM: &str = "publisher.SubscriberDeleteForm";

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

fn publisher_menu() -> Markup {
    let list_url = SubscriberDefaultRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: "Publisher",
        children: sidebar_menu_item_pane(SidebarMenuItem {
            title: "Subscribers",
            url: &list_url,
            active: true,
            ..Default::default()
        }),
    })
}

fn render_pagination<K: SwapKey>(path_and_query: &str, number: u32, num_pages: u32) -> Markup {
    let owned = pagination_pages(path_and_query, number, num_pages, true);
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

lariv_rs::define_register_items! {
    plugin: PublisherTag;
    capability: TemplateCapability;
    trait: TemplateRegistrar;
    method: register_templates;
    wrapper: TemplateOf;
    bounds: [Clone, ProvideRequestCaps, Send, Sync];
    hook: Hook;
    items: [
        SubscriberListIdx: SubscriberListPageTag => SubscriberListPage,
        SubscriberDetailIdx: SubscriberDetailPageTag => SubscriberDetailPage,
        SubscriberCreateModalIdx: SubscriberCreateModalPageTag => SubscriberCreateModalPage,
        SubscriberEditModalIdx: SubscriberEditModalPageTag => SubscriberEditModalPage,
        ConfirmDeleteIdx: PublisherConfirmDeletePageTag => ConfirmDeletePage,
    ]
}

lariv_rs::define_register_items! {
    plugin: PublisherTag;
    capability: SlotCapability;
    trait: SlotRegistrar;
    method: register_slots;
    bounds: [];
    items: [];
    hook: SlotsHook;
}

#[derive(Clone)]
pub struct SubscriberRow {
    pub id: i64,
    pub email: String,
    pub subscription_date: String,
}

#[derive(Generic)]
pub struct SubscriberListPage {
    pub subscribers: ObjectList<SubscriberRow>,
    pub filter_email: String,
    pub sort: String,
    pub path_and_query: String,
    pub can_edit: bool,
}

impl SubscriberListPage {
    pub fn render_table(&self) -> Markup {
        let email_sort = column_sort_url(&self.path_and_query, "Email", &self.sort);
        let email_label = format!("Email{}", sort_indicator(&self.sort, "Email"));
        let date_sort = column_sort_url(&self.path_and_query, "SubscriptionDate", &self.sort);
        let date_label = format!(
            "Subscription date{}",
            sort_indicator(&self.sort, "SubscriptionDate")
        );
        let headers = [
            TableColumnHeader {
                key: "Email",
                label: &email_label,
                sort_url: Some(&email_sort),
                push_url: true,
            },
            TableColumnHeader {
                key: "SubscriptionDate",
                label: &date_label,
                sort_url: Some(&date_sort),
                push_url: true,
            },
        ];
        let rows: Vec<TableRow> = self
            .subscribers
            .items
            .iter()
            .map(|s| TableRow {
                attrs: row_attr_navigate_route(SubscriberDetailRouteTag::new(s.id)),
                cells: vec![
                    field_text(FieldText {
                        value: &s.email,
                        classes: "",
                    }),
                    field_datetime(FieldDatetime {
                        value: &s.subscription_date,
                        classes: "",
                    }),
                ],
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(FormOpts {
                    attrs: form_hx_get_route::<SubscriberTableKey, SubscriberDefaultRouteTag>(
                        SubscriberDefaultRouteTag,
                    ),
                    inputs: SubscriberFilterForm::render_inputs(
                        &FormCtx::form::<SubscriberFilterForm>()
                            .value(SubscriberFilterFormField::Email, &self.filter_email),
                    ),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Apply", ..Default::default() }))
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }))
        };
        if self.can_edit {
            actions = html! {
                (actions)
                (table_create_button::<SubscriberTableKey, SubscriberCreateModalKey>(
                    Some("plus"),
                    "btn-square btn-outline btn-sm",
                ))
            };
        }
        data_table_list_refresh::<SubscriberTableKey>(
            "Subscribers",
            actions,
            &headers,
            &rows,
            render_pagination::<SubscriberTableKey>(
                &self.path_and_query,
                self.subscribers.number,
                self.subscribers.num_pages,
            ),
            &self.path_and_query,
        )
    }
}

impl RenderAppPane for SubscriberListPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            publisher_menu(),
            subscribers_list_crumbs(),
            self.render_table(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(subscribers_list_crumbs(), self.render_table())
    }
}

impl RenderTemplate for SubscriberListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Subscribers — Publisher",
            chrome,
            publisher_menu(),
            subscribers_list_crumbs(),
            self.render_table(),
        )
    }
}

#[derive(Generic)]
pub struct SubscriberDetailPage {
    pub id: i64,
    pub email: String,
    pub subscription_date: String,
    pub can_edit: bool,
}

impl SubscriberDetailPage {
    fn body(&self) -> Markup {
        let actions = if self.can_edit {
            html! {
                (button_modal_form(ButtonModalForm {
                    name: EDIT_FORM,
                    href: &SubscriberEditGetRouteTag::new(self.id).url(),
                    form_post_url: &SubscriberEditPostRouteTag::new(self.id).path(),
                    modal_uid: SubscriberEditModalKey::ID,
                    label: "Edit",
                    classes: "btn-outline",
                    ..Default::default()
                }))
            }
        } else {
            html! {}
        };
        html! {
            (detail(html! {
                (container_column("", html! {
                    (detail_header(DetailHeader {
                        title: &self.email,
                        actions,
                    }))
                    (label("Email", field_text(FieldText {
                        value: &self.email,
                        classes: "",
                    })))
                    (label("Subscription date", field_datetime(FieldDatetime {
                        value: &self.subscription_date,
                        classes: "",
                    })))
                }))
            }))
        }
    }
}

impl RenderAppPane for SubscriberDetailPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            publisher_menu(),
            subscriber_crumbs(&self.email, self.id, None),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(subscriber_crumbs(&self.email, self.id, None), self.body())
    }
}

impl RenderTemplate for SubscriberDetailPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Subscriber — Publisher",
            chrome,
            publisher_menu(),
            subscriber_crumbs(&self.email, self.id, None),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct SubscriberCreateModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub target_input: String,
    pub email: String,
    pub subscription_date: String,
    pub error: String,
}

impl RenderTemplate for SubscriberCreateModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            CREATE_FORM
        } else {
            self.form_name.as_str()
        };
        modal_keyed::<SubscriberCreateModalKey>(
            &self.form_name,
            html! {
                h3 class="font-bold text-lg mb-4" { "New subscriber" }
                (form(FormOpts {
                    attrs: form_hx_post_url::<SubscriberCreateModalKey>(&modal_create_post_query(
                        SubscriberCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                        &self.target_input,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: SubscriberForm::render_inputs(
                        &FormCtx::form::<SubscriberForm>()
                            .value(SubscriberFormField::Email, &self.email)
                            .value(
                                SubscriberFormField::SubscriptionDate,
                                &self.subscription_date,
                            ),
                    ),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Create subscriber", ..Default::default() }))
                    },
                    ..Default::default()
                }))
            },
        )
    }
}

#[derive(Generic)]
pub struct SubscriberEditModalPage {
    pub id: i64,
    pub form_name: String,
    pub email: String,
    pub subscription_date: String,
    pub error: String,
}

impl RenderTemplate for SubscriberEditModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let delete_url = SubscriberDeleteGetRouteTag::new(self.id).url();
        modal_keyed::<SubscriberEditModalKey>(
            &self.form_name,
            html! {
                h3 class="font-bold text-lg mb-4" { "Edit subscriber" }
                (form(FormOpts {
                    attrs: form_hx_post_url::<SubscriberEditModalKey>(&modal_edit_post_url(
                        SubscriberEditPostRouteTag::new(self.id),
                        &self.form_name,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: SubscriberForm::render_inputs(
                        &FormCtx::form::<SubscriberForm>()
                            .value(SubscriberFormField::Email, &self.email)
                            .value(
                                SubscriberFormField::SubscriptionDate,
                                &self.subscription_date,
                            ),
                    ),
                    actions: html! {
                        (button_submit(ButtonSubmit { label: "Save", ..Default::default() }))
                        (button_modal_form(ButtonModalForm {
                            label: "Delete",
                            icon_name: Some("trash"),
                            name: DELETE_FORM,
                            href: &delete_url,
                            form_post_url: &delete_url,
                            modal_uid: SubscriberDeleteModalKey::ID,
                            classes: "btn-error",
                            ..Default::default()
                        }))
                    },
                    ..Default::default()
                }))
            },
        )
    }
}

#[derive(Generic)]
pub struct ConfirmDeletePage {
    pub modal_uid: String,
    pub message: String,
    pub form_name: String,
    pub post_url: String,
    pub error: String,
}

impl RenderTemplate for ConfirmDeletePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let target = format!("#{}", self.modal_uid);
        modal(lariv_rs::components::Modal {
            uid: &self.modal_uid,
            children: delete_confirmation(DeleteConfirmation {
                title: "Confirm Deletion",
                message: &self.message,
                attrs: form_hx_post_selector(&self.post_url, &target),
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
