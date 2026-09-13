use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "publisher_preferences")]
/// Singleton publisher mailing preferences (`id = 1`).
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type PublisherPreferences = Model;
