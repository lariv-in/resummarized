use lariv_rs::html_form::{
    html_form,
    widgets::{CodeEditor, Datetime, ManyToMany, Password, Section, Text},
};

#[html_form]
pub struct SubscriberForm {
    #[form(label = "Email", required, widget = Text)]
    pub email: String,

    #[form(label = "Subscription date", required, widget = Datetime)]
    pub subscription_date: String,
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
