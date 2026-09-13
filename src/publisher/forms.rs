use lariv_rs::html_form::{
    html_form,
    widgets::{
        ChoiceCombobox, CodeEditor, Datetime, Email, List, ManyToMany, Password, Section, Select,
        Text,
    },
};

#[html_form]
pub struct SubscriberForm {
    #[form(label = "Email", required, widget = Text)]
    pub email: String,

    #[form(label = "Subscription date", required, widget = Datetime)]
    pub subscription_date: String,

    #[form(
        label = "Exchanges",
        widget = ChoiceCombobox,
        choices = "exchanges",
        placeholder = "Search exchanges…"
    )]
    pub filter_exchanges: Vec<String>,

    #[form(
        label = "Companies",
        widget = List,
        placeholder = "Symbol or company name"
    )]
    pub filter_entities: Vec<String>,

    #[form(
        label = "Event types",
        widget = List,
        placeholder = "financial-results, announcements…"
    )]
    pub filter_event_types: Vec<String>,

    #[form(
        label = "Newsletter interval",
        required,
        widget = Select,
        choices = "newsletter_interval"
    )]
    pub newsletter_interval: String,
}

#[html_form]
pub struct PublicSubscribeForm {
    #[form(label = "Email", widget = Email, required, name = "email")]
    pub email: String,

    #[form(label = "Exchanges", widget = ChoiceCombobox, name = "filter_exchanges")]
    pub filter_exchanges: Vec<String>,

    #[form(label = "Companies", widget = List, name = "filter_entities")]
    pub filter_entities: Vec<String>,

    #[form(label = "Event types", widget = List, name = "filter_event_types")]
    pub filter_event_types: Vec<String>,

    #[form(label = "Newsletter interval", widget = Text, name = "newsletter_interval")]
    pub newsletter_interval: String,
}

#[html_form]
pub struct PublicEditSubscribeForm {
    #[form(label = "Email", widget = Email, required, name = "email")]
    pub email: String,

    #[form(label = "One-time token", widget = Text, required, name = "one_time_token")]
    pub one_time_token: String,

    #[form(label = "Exchanges", widget = ChoiceCombobox, name = "filter_exchanges")]
    pub filter_exchanges: Vec<String>,

    #[form(label = "Companies", widget = List, name = "filter_entities")]
    pub filter_entities: Vec<String>,

    #[form(label = "Event types", widget = List, name = "filter_event_types")]
    pub filter_event_types: Vec<String>,

    #[form(label = "Newsletter interval", widget = Text, name = "newsletter_interval")]
    pub newsletter_interval: String,
}

#[html_form]
pub struct PublicCancelSubscribeForm {
    #[form(label = "Email", widget = Email, required, name = "email")]
    pub email: String,

    #[form(label = "One-time token", widget = Text, required, name = "one_time_token")]
    pub one_time_token: String,
}

#[html_form]
pub struct SubscriberFilterForm {
    #[form(label = "Email", widget = Text)]
    pub email: String,
}

#[html_form(default)]
pub struct PreferencesForm {
    #[form(widget = Section, label = "Email template")]
    _section_template: (),

    #[form(
        label = "HTML template",
        widget = CodeEditor,
        language = "html",
        rows = 20
    )]
    pub html_template: String,

    #[form(widget = Section, label = "SMTP")]
    _section_smtp: (),

    #[form(label = "SMTP host", widget = Text, row = "smtp_host")]
    pub smtp_host: String,

    #[form(label = "SMTP port", widget = Text, row = "smtp_host")]
    pub smtp_port: String,

    #[form(label = "SMTP username", widget = Text, row = "smtp_user")]
    pub smtp_username: String,

    #[form(label = "SMTP password", widget = Password, row = "smtp_user")]
    pub smtp_password: String,

    #[form(label = "SMTP from address", widget = Text)]
    pub smtp_from: String,

    #[form(widget = Section, label = "Subscription Change Email")]
    _section_edit_link: (),

    #[form(
        label = "Change email subject",
        widget = Text,
        placeholder = "Change your Resummarized subscription preferences"
    )]
    pub edit_link_email_subject: String,

    #[form(
        label = "Change email HTML template",
        widget = CodeEditor,
        language = "html",
        rows = 12
    )]
    pub edit_link_email_template: String,

    #[form(widget = Section, label = "Subscription Link Opened Notification")]
    _section_edit_opened: (),

    #[form(
        label = "Link opened subject",
        widget = Text,
        placeholder = "Your Resummarized subscription preferences link"
    )]
    pub edit_opened_email_subject: String,

    #[form(
        label = "Link opened HTML template",
        widget = CodeEditor,
        language = "html",
        rows = 12
    )]
    pub edit_opened_email_template: String,

    #[form(widget = Section, label = "Subscription Updated Notification")]
    _section_edit_updated: (),

    #[form(
        label = "Updated subject",
        widget = Text,
        placeholder = "Your Resummarized subscription preferences were updated"
    )]
    pub edit_updated_email_subject: String,

    #[form(
        label = "Updated HTML template",
        widget = CodeEditor,
        language = "html",
        rows = 12
    )]
    pub edit_updated_email_template: String,
}

#[html_form]
pub struct SendEmailForm {
    #[form(label = "Subject", required, widget = Text)]
    pub subject: String,

    #[form(
        label = "Attachments",
        widget = ManyToMany,
        url = "/filesystem/file-select/",
        swap_key = "fk-publisher-email-attachments",
        placeholder = "Select files..."
    )]
    pub attachments: Vec<i64>,
}
