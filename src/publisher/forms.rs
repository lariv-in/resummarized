use lariv_rs::html_form::{
    html_form,
    widgets::{Datetime, Text},
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
