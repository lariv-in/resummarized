//! Singleton publisher preferences (`id = 1`).

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};

use super::entities::{
    PublisherPreferences,
    publisher_preferences::{self, Entity as PrefsEntity},
};

use super::email::DEFAULT_EMAIL_HTML_TEMPLATE;

const DEFAULT_HTML_TEMPLATE: &str = DEFAULT_EMAIL_HTML_TEMPLATE;

/// Load singleton preferences row (`id = 1`), creating it if missing.
pub async fn load_preferences(
    db: &DatabaseConnection,
) -> Result<PublisherPreferences, sea_orm::DbErr> {
    if let Some(prefs) = PrefsEntity::find_by_id(1).one(db).await? {
        return Ok(prefs);
    }

    let now = Utc::now();
    let model = publisher_preferences::ActiveModel {
        id: Set(1),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        html_template: Set(DEFAULT_HTML_TEMPLATE.to_string()),
        smtp_host: Set(String::new()),
        smtp_port: Set(String::new()),
        smtp_username: Set(String::new()),
        smtp_password: Set(String::new()),
        smtp_from: Set(String::new()),
        edit_link_email_subject: Set(String::new()),
        edit_link_email_template: Set(String::new()),
        edit_opened_email_subject: Set(String::new()),
        edit_opened_email_template: Set(String::new()),
        edit_updated_email_subject: Set(String::new()),
        edit_updated_email_template: Set(String::new()),
    };
    model.insert(db).await
}

/// Persist preferences fields onto the singleton row.
pub async fn save_preferences(
    db: &DatabaseConnection,
    prefs: PublisherPreferences,
) -> Result<PublisherPreferences, sea_orm::DbErr> {
    let mut am: publisher_preferences::ActiveModel = load_preferences(db).await?.into();
    am.html_template = Set(prefs.html_template);
    am.smtp_host = Set(prefs.smtp_host);
    am.smtp_port = Set(prefs.smtp_port);
    am.smtp_username = Set(prefs.smtp_username);
    am.smtp_password = Set(prefs.smtp_password);
    am.smtp_from = Set(prefs.smtp_from);
    am.edit_link_email_subject = Set(prefs.edit_link_email_subject);
    am.edit_link_email_template = Set(prefs.edit_link_email_template);
    am.edit_opened_email_subject = Set(prefs.edit_opened_email_subject);
    am.edit_opened_email_template = Set(prefs.edit_opened_email_template);
    am.edit_updated_email_subject = Set(prefs.edit_updated_email_subject);
    am.edit_updated_email_template = Set(prefs.edit_updated_email_template);
    am.updated_at = Set(Some(Utc::now()));
    am.update(db).await
}

pub fn empty_preferences() -> PublisherPreferences {
    PublisherPreferences {
        id: 1,
        created_at: None,
        updated_at: None,
        html_template: DEFAULT_HTML_TEMPLATE.to_string(),
        smtp_host: String::new(),
        smtp_port: String::new(),
        smtp_username: String::new(),
        smtp_password: String::new(),
        smtp_from: String::new(),
        edit_link_email_subject: String::new(),
        edit_link_email_template: String::new(),
        edit_opened_email_subject: String::new(),
        edit_opened_email_template: String::new(),
        edit_updated_email_subject: String::new(),
        edit_updated_email_template: String::new(),
    }
}
