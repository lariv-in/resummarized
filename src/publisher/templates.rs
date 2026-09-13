use frunk::Generic;
use lariv_rs::{
    components::{
        ButtonModalForm, ButtonSubmit, DeleteConfirmation, DetailHeader, FieldDatetime, FieldText,
        FormOpts, InputText, InputTextarea, LayoutMain, LayoutSidebar, ManyToManyItem, ObjectList,
        PaginationPage, ShellChrome, ShellScaffold, SidebarMenu, SidebarNavLink, SlotCapability,
        SlotRegistrar, SwapKey, TableButtonFilter, TableColumnHeader, TablePagination, TableRow,
        button_modal_form, button_submit, column_sort_url, container_column,
        data_table_list_refresh, delete_confirmation, detail, detail_header, field_datetime,
        field_text, form, form_hx_get_route, form_hx_post_main, form_hx_post_selector,
        form_hx_post_url, input_text, input_textarea, label, layout_main, layout_sidebar, modal,
        modal_keyed, pagination_pages, row_attr_navigate_route, shell_scaffold, sidebar_menu,
        sidebar_nav_items_pane, sort_indicator, table_button_bulk_actions, table_button_filter,
        table_create_button, table_pagination,
    },
    html_form::{CsrfToken, FormCtx, HtmlForm, csrf_hidden_field},
    http::ProvideRequestCaps,
    template::{RenderAppPane, RenderTemplate, TemplateCapability, TemplateOf, TemplateRegistrar},
    web::{modal_create_post_query, modal_create_post_url, modal_edit_post_url},
};
use maud::{Markup, PreEscaped, html};

use super::catalog;
use super::crumbs::{preferences_crumbs, subscriber_crumbs, subscribers_list_crumbs};
use super::email::TemplateContextField;
use super::forms::{
    PreferencesForm, PreferencesFormField, SendEmailForm, SendEmailFormField, SubscriberFilterForm,
    SubscriberFilterFormField, SubscriberForm, SubscriberFormField,
};
use super::keys::{
    SubscriberBulkDeleteModalKey, SubscriberBulkSendEditLinkModalKey, SubscriberCreateModalKey,
    SubscriberDeleteModalKey, SubscriberEditModalKey, SubscriberSendEditLinkModalKey,
    SubscriberSendEmailModalKey, SubscriberTableKey,
};
use super::routes::{
    PublisherPrefsGetRouteTag, PublisherPrefsPostRouteTag, SubscriberBulkDeletePostRouteTag,
    SubscriberBulkSendEditLinkPostRouteTag, SubscriberCreatePostRouteTag, SubscriberDefaultRouteTag,
    SubscriberDeleteGetRouteTag, SubscriberDetailRouteTag, SubscriberEditGetRouteTag,
    SubscriberEditPostRouteTag, SubscriberSendEditLinkGetRouteTag,
    SubscriberSendEditLinkPostRouteTag, SubscriberSendEmailPostRouteTag,
};

const CREATE_FORM: &str = "publisher.SubscriberCreateForm";
const EDIT_FORM: &str = "publisher.SubscriberEditForm";
const DELETE_FORM: &str = "publisher.SubscriberDeleteForm";
const SEND_EMAIL_FORM: &str = "publisher.SubscriberSendEmailForm";

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

fn publisher_menu(current_path: &str) -> Markup {
    let list_url = SubscriberDefaultRouteTag.url();
    let prefs_url = PublisherPrefsGetRouteTag.url();
    let links = [
        SidebarNavLink {
            key: "subscribers",
            title: "Subscribers",
            url: &list_url,
            icon_name: None,
            match_prefixes: &["/publisher/subscribers"],
        },
        SidebarNavLink {
            key: "preferences",
            title: "Preferences",
            url: &prefs_url,
            icon_name: None,
            match_prefixes: &[],
        },
    ];
    sidebar_menu(SidebarMenu {
        title: "Publisher",
        children: sidebar_nav_items_pane(&links, current_path),
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
        ConfirmBulkDeleteIdx: PublisherConfirmBulkDeletePageTag => ConfirmBulkDeletePage,
        PreferencesIdx: PublisherPreferencesPageTag => PublisherPreferencesPage,
        SendEmailModalIdx: SubscriberSendEmailModalPageTag => SubscriberSendEmailModalPage,
        SendEditLinkModalIdx: SubscriberSendEditLinkModalPageTag => SubscriberSendEditLinkModalPage,
        BulkSendEditLinkModalIdx: SubscriberBulkSendEditLinkModalPageTag => SubscriberBulkSendEditLinkModalPage,
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
    pub newsletter_interval: String,
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
    fn selection_root_js() -> &'static str {
        "Alpine.$data($el.closest('[data-publisher-selection]'))"
    }

    fn selection_x_data() -> &'static str {
        r#"{
            selected: {},
            toggle(id) {
                const k = String(id);
                if (this.selected[k]) delete this.selected[k];
                else this.selected[k] = true;
            },
            setVisible(ids, on) {
                for (const id of ids) {
                    const k = String(id);
                    if (on) this.selected[k] = true;
                    else delete this.selected[k];
                }
            },
            allVisibleSelected(ids) {
                return ids.length > 0 && ids.every(id => !!this.selected[String(id)]);
            },
            someVisibleSelected(ids) {
                return ids.some(id => !!this.selected[String(id)]);
            },
            selectedIds() {
                return Object.keys(this.selected).filter(k => this.selected[k]);
            },
            bulkSendHref() {
                const ids = this.selectedIds();
                if (ids.length < 1) return '#';
                return '/publisher/subscribers/send-email/?ids=' + ids.join(',');
            },
            bulkSendAllHref() {
                return '/publisher/subscribers/send-email/?all=1';
            },
            requestBulkSend(el) {
                const href = this.bulkSendHref();
                if (href === '#' || typeof htmx === 'undefined') return;
                htmx.ajax('GET', href, { target: 'body', swap: 'beforeend', source: el });
            },
            requestBulkSendAll(el) {
                if (typeof htmx === 'undefined') return;
                htmx.ajax('GET', this.bulkSendAllHref(), { target: 'body', swap: 'beforeend', source: el });
            },
            bulkDeleteHref() {
                const ids = this.selectedIds();
                if (ids.length < 1) return '#';
                return '/publisher/subscribers/bulk-delete/?ids=' + ids.join(',');
            },
            requestBulkDelete(el) {
                const href = this.bulkDeleteHref();
                if (href === '#' || typeof htmx === 'undefined') return;
                htmx.ajax('GET', href, { target: 'body', swap: 'beforeend', source: el });
            },
            bulkSendEditLinkHref() {
                const ids = this.selectedIds();
                if (ids.length < 1) return '#';
                return '/publisher/subscribers/bulk-send-edit-link/?ids=' + ids.join(',');
            },
            requestBulkSendEditLink(el) {
                const href = this.bulkSendEditLinkHref();
                if (href === '#' || typeof htmx === 'undefined') return;
                htmx.ajax('GET', href, { target: 'body', swap: 'beforeend', source: el });
            }
        }"#
    }

    fn wrap_with_selection(&self, table: Markup) -> Markup {
        html! {
            (PreEscaped(format!(
                r#"<div data-publisher-selection x-data="{}">"#,
                lariv_rs::components::attrs::escape_attr(Self::selection_x_data()),
            )))
            (table)
            (PreEscaped("</div>"))
        }
    }

    fn body(&self) -> Markup {
        if self.can_edit {
            self.wrap_with_selection(self.render_table())
        } else {
            self.render_table()
        }
    }

    pub fn render_table(&self) -> Markup {
        let sel = Self::selection_root_js();
        let email_sort = column_sort_url(&self.path_and_query, "Email", &self.sort);
        let email_label = format!("Email{}", sort_indicator(&self.sort, "Email"));
        let date_sort = column_sort_url(&self.path_and_query, "SubscriptionDate", &self.sort);
        let date_label = format!(
            "Subscription date{}",
            sort_indicator(&self.sort, "SubscriptionDate")
        );
        let interval_sort = column_sort_url(&self.path_and_query, "NewsletterInterval", &self.sort);
        let interval_label = format!(
            "Interval{}",
            sort_indicator(&self.sort, "NewsletterInterval")
        );
        let visible_ids: Vec<i64> = self.subscribers.items.iter().map(|s| s.id).collect();
        let visible_ids_js = format!(
            "[{}]",
            visible_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let select_all_label = format!(
            r#"<label class="flex justify-center" @click.stop=""><input type="checkbox" class="checkbox checkbox-sm" @change="{sel}.setVisible({ids}, $event.target.checked)" :checked="{sel}.allVisibleSelected({ids})" x-effect="$el.indeterminate = {sel}.someVisibleSelected({ids}) && !{sel}.allVisibleSelected({ids})" /></label>"#,
            sel = sel,
            ids = visible_ids_js,
        );
        let mut headers = Vec::new();
        if self.can_edit {
            headers.push(TableColumnHeader {
                key: "Select",
                label: &select_all_label,
                sort_url: None,
                push_url: true,
            });
        }
        headers.push(TableColumnHeader {
            key: "Email",
            label: &email_label,
            sort_url: Some(&email_sort),
            push_url: true,
        });
        headers.push(TableColumnHeader {
            key: "SubscriptionDate",
            label: &date_label,
            sort_url: Some(&date_sort),
            push_url: true,
        });
        headers.push(TableColumnHeader {
            key: "NewsletterInterval",
            label: &interval_label,
            sort_url: Some(&interval_sort),
            push_url: true,
        });
        let rows: Vec<TableRow> = self
            .subscribers
            .items
            .iter()
            .map(|s| {
                let mut cells = Vec::new();
                if self.can_edit {
                    cells.push(
                        PreEscaped(format!(
                            r#"<label class="flex justify-center" @click.stop=""><input type="checkbox" class="checkbox checkbox-sm" @change="{sel}.toggle({id})" :checked="!!{sel}.selected['{id}']" /></label>"#,
                            sel = sel,
                            id = s.id,
                        ))
                        .into(),
                    );
                }
                cells.push(field_text(FieldText {
                    value: &s.email,
                    classes: "",
                }));
                cells.push(field_datetime(FieldDatetime {
                    value: &s.subscription_date,
                    classes: "",
                }));
                cells.push(field_text(FieldText {
                    value: &s.newsletter_interval,
                    classes: "",
                }));
                TableRow {
                    attrs: row_attr_navigate_route(SubscriberDetailRouteTag::new(s.id)),
                    cells,
                }
            })
            .collect();
        let mut actions = html! {
            (table_button_filter(TableButtonFilter {
                panel: form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_get_route::<SubscriberTableKey, SubscriberDefaultRouteTag>(
                        SubscriberDefaultRouteTag,
                    ),
                    inputs: SubscriberFilterForm::render_inputs(
                        &FormCtx::form::<SubscriberFilterForm>(CsrfToken::current())
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
            let bulk_item = |label: &str,
                             classes: &str,
                             on_click: &str,
                             require_selection: bool| {
                let disabled = if require_selection {
                    format!(
                        r#" x-bind:class="{sel}.selectedIds().length >= 1 ? '' : 'btn-disabled pointer-events-none opacity-50'""#,
                        sel = sel,
                    )
                } else {
                    String::new()
                };
                format!(
                    r#"<button type="button" class="btn {classes} btn-sm justify-start w-full"{disabled} @click="{sel}.{on_click}($el); $el.closest('details')?.removeAttribute('open')">{label}</button>"#,
                    classes = classes,
                    sel = sel,
                    on_click = on_click,
                    label = label,
                    disabled = disabled,
                )
            };
            let bulk_items = format!(
                "{}{}{}{}",
                bulk_item("Send email", "btn-ghost", "requestBulkSend", true),
                bulk_item(
                    "Send to all subscribers",
                    "btn-ghost",
                    "requestBulkSendAll",
                    false
                ),
                bulk_item(
                    "Send subscription edit link",
                    "btn-ghost",
                    "requestBulkSendEditLink",
                    true
                ),
                bulk_item(
                    "Delete selected",
                    "btn-ghost text-error",
                    "requestBulkDelete",
                    true
                ),
            );
            let bulk_actions = table_button_bulk_actions(html! {
                (PreEscaped(bulk_items))
            });
            actions = html! {
                (actions)
                (bulk_actions)
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
            publisher_menu(&SubscriberDefaultRouteTag.url()),
            subscribers_list_crumbs(),
            self.body(),
        )
    }
    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(subscribers_list_crumbs(), self.body())
    }
}

impl RenderTemplate for SubscriberListPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Subscribers — Publisher",
            chrome,
            publisher_menu(&SubscriberDefaultRouteTag.url()),
            subscribers_list_crumbs(),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct SubscriberDetailPage {
    pub id: i64,
    pub email: String,
    pub subscription_date: String,
    pub filter_exchanges: String,
    pub filter_entities: String,
    pub filter_event_types: String,
    pub newsletter_interval: String,
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
                (button_modal_form(ButtonModalForm {
                    name: "send_edit_link",
                    href: &SubscriberSendEditLinkGetRouteTag::new(self.id).url(),
                    form_post_url: &SubscriberSendEditLinkPostRouteTag::new(self.id).path(),
                    modal_uid: SubscriberSendEditLinkModalKey::ID,
                    label: "Send Edit Link",
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
                    (label("Newsletter interval", field_text(FieldText {
                        value: &self.newsletter_interval,
                        classes: "",
                    })))
                    (label("Exchanges", field_text(FieldText {
                        value: &self.filter_exchanges,
                        classes: "",
                    })))
                    (label("Companies", field_text(FieldText {
                        value: &self.filter_entities,
                        classes: "",
                    })))
                    (label("Event types", field_text(FieldText {
                        value: &self.filter_event_types,
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
            publisher_menu(&SubscriberDetailRouteTag::new(self.id).url()),
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
            publisher_menu(&SubscriberDetailRouteTag::new(self.id).url()),
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
    pub filter_exchanges: Vec<String>,
    pub filter_entities: Vec<String>,
    pub filter_event_types: Vec<String>,
    pub newsletter_interval: String,
    pub error: String,
}

fn subscriber_form_inputs(
    email: &str,
    subscription_date: &str,
    filter_exchanges: &[String],
    filter_entities: &[String],
    filter_event_types: &[String],
    newsletter_interval: &str,
) -> Markup {
    let interval_choices = catalog::newsletter_interval_choices();
    let exchange_choices = catalog::exchange_choices();
    SubscriberForm::render_inputs(
        &FormCtx::form::<SubscriberForm>(CsrfToken::current())
            .value(SubscriberFormField::Email, email)
            .value(SubscriberFormField::SubscriptionDate, subscription_date)
            .value(SubscriberFormField::NewsletterInterval, newsletter_interval)
            .list(SubscriberFormField::FilterExchanges, filter_exchanges)
            .list(SubscriberFormField::FilterEntities, filter_entities)
            .list(SubscriberFormField::FilterEventTypes, filter_event_types)
            .choices(SubscriberFormField::FilterExchanges, &exchange_choices)
            .choices(SubscriberFormField::NewsletterInterval, &interval_choices)
            .hint(
                SubscriberFormField::FilterExchanges,
                "Leave empty to include all exchanges.",
            )
            .hint(
                SubscriberFormField::FilterEntities,
                "Leave empty to include all companies.",
            )
            .hint(
                SubscriberFormField::FilterEventTypes,
                "Leave empty to include all event types. Use feed slugs such as financial-results.",
            ),
    )
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
                (form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_post_url::<SubscriberCreateModalKey>(&modal_create_post_query(
                        SubscriberCreatePostRouteTag,
                        form_name,
                        &self.refresh_table,
                        &self.target_input,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: subscriber_form_inputs(
                        &self.email,
                        &self.subscription_date,
                        &self.filter_exchanges,
                        &self.filter_entities,
                        &self.filter_event_types,
                        &self.newsletter_interval,
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
    pub filter_exchanges: Vec<String>,
    pub filter_entities: Vec<String>,
    pub filter_event_types: Vec<String>,
    pub newsletter_interval: String,
    pub error: String,
}

impl RenderTemplate for SubscriberEditModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let delete_url = SubscriberDeleteGetRouteTag::new(self.id).url();
        modal_keyed::<SubscriberEditModalKey>(
            &self.form_name,
            html! {
                h3 class="font-bold text-lg mb-4" { "Edit subscriber" }
                (form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_post_url::<SubscriberEditModalKey>(&modal_edit_post_url(
                        SubscriberEditPostRouteTag::new(self.id),
                        &self.form_name,
                    )),
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: subscriber_form_inputs(
                        &self.email,
                        &self.subscription_date,
                        &self.filter_exchanges,
                        &self.filter_entities,
                        &self.filter_event_types,
                        &self.newsletter_interval,
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

#[derive(Generic)]
pub struct ConfirmBulkDeletePage {
    pub modal_uid: String,
    pub message: String,
    pub form_name: String,
    pub ids: String,
    pub error: String,
    pub can_submit: bool,
}

impl RenderTemplate for ConfirmBulkDeletePage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let target = if self.modal_uid.is_empty() {
            format!("#{}", SubscriberBulkDeleteModalKey::ID)
        } else {
            format!("#{}", self.modal_uid)
        };
        let uid = if self.modal_uid.is_empty() {
            SubscriberBulkDeleteModalKey::ID
        } else {
            self.modal_uid.as_str()
        };
        let post_url = SubscriberBulkDeletePostRouteTag.url();
        let form_attrs = form_hx_post_selector(&post_url, &target);
        modal(lariv_rs::components::Modal {
            uid,
            children: html! {
                div class="container mx-auto" {
                    h2 class="text-xl font-bold text-error" { "Confirm Deletion" }
                    p class="my-2" { (self.message) }
                    @if !self.error.is_empty() {
                        div class="alert alert-error my-2 text-sm" { (self.error) }
                    }
                    @if self.can_submit {
                        (PreEscaped(format!(
                            r#"<form class="flex flex-col gap-2 my-4"{}>"#,
                            form_attrs.as_string(),
                        )))
                        (csrf_hidden_field(&CsrfToken::current()))
                        input type="hidden" name="ids" value=(self.ids);
                        div class="my-2" {
                            (button_submit(ButtonSubmit {
                                label: "Confirm Delete",
                                classes: "btn-error my-2",
                                ..Default::default()
                            }))
                        }
                        (PreEscaped("</form>"))
                    }
                }
            },
            ..Default::default()
        })
    }
}

#[derive(Generic)]
pub struct SubscriberSendEditLinkModalPage {
    pub modal_uid: String,
    pub subscriber_id: i64,
    pub email: String,
    pub error: String,
}

impl RenderTemplate for SubscriberSendEditLinkModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let target = if self.modal_uid.is_empty() {
            format!("#{}", SubscriberSendEditLinkModalKey::ID)
        } else {
            format!("#{}", self.modal_uid)
        };
        let uid = if self.modal_uid.is_empty() {
            SubscriberSendEditLinkModalKey::ID
        } else {
            self.modal_uid.as_str()
        };
        let post_url = SubscriberSendEditLinkPostRouteTag::new(self.subscriber_id).url();
        let form_attrs = form_hx_post_selector(&post_url, &target);
        modal(lariv_rs::components::Modal {
            uid,
            children: html! {
                div class="container mx-auto" {
                    h2 class="text-xl font-bold" { "Send Subscription Change Email" }
                    p class="my-2" {
                        "Are you sure you want to send a subscription change email with a secure link to "
                        strong { (self.email) }
                        "?"
                    }
                    @if !self.error.is_empty() {
                        div class="alert alert-error my-2 text-sm" { (self.error) }
                    }
                    (PreEscaped(format!(
                        r#"<form class="flex flex-col gap-2 my-4"{}>"#,
                        form_attrs.as_string(),
                    )))
                    (csrf_hidden_field(&CsrfToken::current()))
                    div class="my-2" {
                        (button_submit(ButtonSubmit {
                            label: "Send Link",
                            classes: "btn-primary my-2",
                            ..Default::default()
                        }))
                    }
                    (PreEscaped("</form>"))
                }
            },
            ..Default::default()
        })
    }
}

#[derive(Generic)]
pub struct SubscriberBulkSendEditLinkModalPage {
    pub modal_uid: String,
    pub ids: String,
    pub recipient_count: usize,
    pub error: String,
    pub can_submit: bool,
}

impl RenderTemplate for SubscriberBulkSendEditLinkModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let target = if self.modal_uid.is_empty() {
            format!("#{}", SubscriberBulkSendEditLinkModalKey::ID)
        } else {
            format!("#{}", self.modal_uid)
        };
        let uid = if self.modal_uid.is_empty() {
            SubscriberBulkSendEditLinkModalKey::ID
        } else {
            self.modal_uid.as_str()
        };
        let post_url = SubscriberBulkSendEditLinkPostRouteTag.url();
        let form_attrs = form_hx_post_selector(&post_url, &target);
        modal(lariv_rs::components::Modal {
            uid,
            children: html! {
                div class="container mx-auto" {
                    h2 class="text-xl font-bold" { "Send Subscription Change Emails" }
                    p class="my-2" {
                        "Send subscription change emails with unique links to "
                        strong { (self.recipient_count) }
                        " selected subscriber(s)?"
                    }
                    @if !self.error.is_empty() {
                        div class="alert alert-error my-2 text-sm" { (self.error) }
                    }
                    @if self.can_submit {
                        (PreEscaped(format!(
                            r#"<form class="flex flex-col gap-2 my-4"{}>"#,
                            form_attrs.as_string(),
                        )))
                        (csrf_hidden_field(&CsrfToken::current()))
                        input type="hidden" name="ids" value=(self.ids);
                        div class="my-2" {
                            (button_submit(ButtonSubmit {
                                label: "Send Emails",
                                classes: "btn-primary my-2",
                                ..Default::default()
                            }))
                        }
                        (PreEscaped("</form>"))
                    }
                }
            },
            ..Default::default()
        })
    }
}

#[derive(Generic)]
pub struct PublisherPreferencesPage {
    pub html_template: String,
    pub smtp_host: String,
    pub smtp_port: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub edit_link_email_subject: String,
    pub edit_link_email_template: String,
    pub edit_opened_email_subject: String,
    pub edit_opened_email_template: String,
    pub edit_updated_email_subject: String,
    pub edit_updated_email_template: String,
    pub error: String,
}

impl PublisherPreferencesPage {
    fn body(&self) -> Markup {
        form(
            &CsrfToken::current(),
            FormOpts {
                attrs: form_hx_post_main(PublisherPrefsPostRouteTag),
                title: "Publisher Preferences",
                subtitle: "HTML email template, SMTP settings, and subscription change email templates",
                form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                inputs: PreferencesForm::render_inputs(
                    &FormCtx::form::<PreferencesForm>(CsrfToken::current())
                        .value(
                            PreferencesFormField::HtmlTemplate,
                            self.html_template.as_str(),
                        )
                        .value(PreferencesFormField::SmtpHost, self.smtp_host.as_str())
                        .value(PreferencesFormField::SmtpPort, self.smtp_port.as_str())
                        .value(
                            PreferencesFormField::SmtpUsername,
                            self.smtp_username.as_str(),
                        )
                        .value(
                            PreferencesFormField::SmtpPassword,
                            self.smtp_password.as_str(),
                        )
                        .value(PreferencesFormField::SmtpFrom, self.smtp_from.as_str())
                        .value(
                            PreferencesFormField::EditLinkEmailSubject,
                            self.edit_link_email_subject.as_str(),
                        )
                        .value(
                            PreferencesFormField::EditLinkEmailTemplate,
                            self.edit_link_email_template.as_str(),
                        )
                        .value(
                            PreferencesFormField::EditOpenedEmailSubject,
                            self.edit_opened_email_subject.as_str(),
                        )
                        .value(
                            PreferencesFormField::EditOpenedEmailTemplate,
                            self.edit_opened_email_template.as_str(),
                        )
                        .value(
                            PreferencesFormField::EditUpdatedEmailSubject,
                            self.edit_updated_email_subject.as_str(),
                        )
                        .value(
                            PreferencesFormField::EditUpdatedEmailTemplate,
                            self.edit_updated_email_template.as_str(),
                        ),
                ),
                actions: html! {
                    (button_submit(ButtonSubmit {
                        label: "Save Preferences",
                        ..Default::default()
                    }))
                },
                ..Default::default()
            },
        )
    }
}

impl RenderAppPane for PublisherPreferencesPage {
    fn render_pane(&self) -> lariv_rs::components::AppLayoutHtml {
        scaffold_pane(
            publisher_menu(&PublisherPrefsGetRouteTag.url()),
            preferences_crumbs(),
            self.body(),
        )
    }

    fn render_main(&self) -> lariv_rs::components::MainContentHtml {
        scaffold_main(preferences_crumbs(), self.body())
    }
}

impl RenderTemplate for PublisherPreferencesPage {
    fn render(&self, chrome: &ShellChrome) -> Markup {
        app_scaffold(
            "Preferences — Publisher",
            chrome,
            publisher_menu(&PublisherPrefsGetRouteTag.url()),
            preferences_crumbs(),
            self.body(),
        )
    }
}

#[derive(Generic)]
pub struct SubscriberSendEmailModalPage {
    pub form_name: String,
    pub refresh_table: String,
    pub ids: String,
    pub send_all: bool,
    pub recipient_count: usize,
    pub subject: String,
    pub attachments: Vec<ManyToManyItem>,
    pub context_fields: Vec<TemplateContextField>,
    pub error: String,
    pub can_submit: bool,
}

impl RenderTemplate for SubscriberSendEmailModalPage {
    fn render(&self, _chrome: &ShellChrome) -> Markup {
        let form_name = if self.form_name.is_empty() {
            SEND_EMAIL_FORM
        } else {
            self.form_name.as_str()
        };
        let mut post_url = modal_create_post_url(
            SubscriberSendEmailPostRouteTag,
            form_name,
            &self.refresh_table,
        );
        if self.send_all {
            post_url.push_str("&all=1");
        } else if !self.ids.is_empty() {
            post_url.push_str("&ids=");
            post_url.push_str(&self.ids);
        }
        let subtitle = if self.send_all {
            format!(
                "Send the preference HTML template to all {} subscriber(s). Attach files from the filesystem.",
                self.recipient_count
            )
        } else if self.recipient_count == 1 {
            "Send the preference HTML template to the selected subscriber. Attach files from the filesystem.".to_string()
        } else {
            format!(
                "Send the preference HTML template to {} selected subscribers. Attach files from the filesystem.",
                self.recipient_count
            )
        };
        let submit = if self.can_submit {
            html! {
                (button_submit(ButtonSubmit { label: "Send email", ..Default::default() }))
            }
        } else {
            html! {}
        };
        modal_keyed::<SubscriberSendEmailModalKey>(
            &self.form_name,
            html! {
                h3 class="font-bold text-lg mb-4" { "Send email" }
                (form(&CsrfToken::current(), FormOpts {
                    attrs: form_hx_post_url::<SubscriberSendEmailModalKey>(&post_url),
                    subtitle: &subtitle,
                    form_error: Some(self.error.as_str()).filter(|e| !e.is_empty()),
                    inputs: html! {
                        (SendEmailForm::render_inputs(
                            &FormCtx::form::<SendEmailForm>(CsrfToken::current())
                                .value(SendEmailFormField::Subject, &self.subject)
                                .m2m(SendEmailFormField::Attachments, &self.attachments),
                        ))
                        @if !self.context_fields.is_empty() {
                            (field_text(FieldText {
                                value: "Template values",
                                classes: "text-lg font-semibold mt-4",
                            }))
                            p class="text-sm text-gray-500 mb-1" {
                                "Taken from placeholders in the HTML template (and subject)."
                            }
                            @for field in &self.context_fields {
                                @if field.multiline {
                                    (input_textarea(InputTextarea {
                                        label: &field.label,
                                        name: &field.input_name,
                                        value: &field.value,
                                        rows: 4,
                                        hint: field.hint.as_deref(),
                                        ..Default::default()
                                    }))
                                } @else {
                                    (input_text(InputText {
                                        label: &field.label,
                                        name: &field.input_name,
                                        value: &field.value,
                                        ..Default::default()
                                    }))
                                }
                            }
                        }
                    },
                    actions: submit,
                    ..Default::default()
                }))
            },
        )
    }
}
