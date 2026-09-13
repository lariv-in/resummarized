use lariv_rs::plugins::users::state::AuthContext;
use lariv_rs::web::opt_or_log;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select, sea_query::Expr,
};

use super::entities::subscriber::{self, Entity as SubscriberEntity};

pub fn scope_superuser<T>(query: Select<T>, auth: &AuthContext) -> Select<T>
where
    T: EntityTrait,
{
    if auth.user.is_superuser {
        return query;
    }
    query.filter(Expr::cust("1 = 0"))
}

pub fn apply_email_filter(
    mut query: Select<SubscriberEntity>,
    email: Option<&str>,
) -> Select<SubscriberEntity> {
    if let Some(e) = email.filter(|s| !s.is_empty()) {
        query = query.filter(subscriber::Column::Email.contains(e));
    }
    query
}

fn sort_desc(sort: Option<&str>, key: &str) -> Option<bool> {
    let s = sort.unwrap_or("").trim();
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts
        .first()
        .is_none_or(|col| !col.eq_ignore_ascii_case(key))
    {
        return None;
    }
    Some(parts.get(1).is_some_and(|d| d.eq_ignore_ascii_case("DESC")))
}

pub fn apply_subscriber_sort(
    query: Select<SubscriberEntity>,
    sort: Option<&str>,
) -> Select<SubscriberEntity> {
    if let Some(desc) = sort_desc(sort, "Email") {
        if desc {
            query.order_by_desc(subscriber::Column::Email)
        } else {
            query.order_by_asc(subscriber::Column::Email)
        }
    } else if let Some(desc) = sort_desc(sort, "SubscriptionDate") {
        if desc {
            query.order_by_desc(subscriber::Column::SubscriptionDate)
        } else {
            query.order_by_asc(subscriber::Column::SubscriptionDate)
        }
    } else if let Some(desc) = sort_desc(sort, "NewsletterInterval") {
        if desc {
            query.order_by_desc(subscriber::Column::NewsletterInterval)
        } else {
            query.order_by_asc(subscriber::Column::NewsletterInterval)
        }
    } else {
        query.order_by_desc(subscriber::Column::Id)
    }
}

pub async fn find_subscriber_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<subscriber::Model> {
    opt_or_log(
        scope_superuser(SubscriberEntity::find_by_id(id), auth)
            .one(db)
            .await,
        "find subscriber",
    )
}

pub fn normalize_subscriber_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub fn email_looks_valid(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.contains(' ')
}

pub fn is_unique_violation(err: &sea_orm::DbErr) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("unique") || msg.contains("duplicate")
}

pub async fn email_in_use(db: &DatabaseConnection, email: &str, exclude_id: Option<i64>) -> bool {
    let email = normalize_subscriber_email(email);
    let mut query = SubscriberEntity::find().filter(subscriber::Column::Email.eq(email));
    if let Some(id) = exclude_id {
        query = query.filter(subscriber::Column::Id.ne(id));
    }
    opt_or_log(query.one(db).await, "check subscriber email").is_some()
}

#[cfg(test)]
mod tests {
    use super::{email_looks_valid, normalize_subscriber_email};

    #[test]
    fn normalize_trims_and_lowercases() {
        assert_eq!(
            normalize_subscriber_email("  Foo.Bar@Example.COM "),
            "foo.bar@example.com"
        );
    }

    #[test]
    fn email_looks_valid_accepts_simple_addresses() {
        assert!(email_looks_valid("reader@example.com"));
        assert!(!email_looks_valid(""));
        assert!(!email_looks_valid("no-at-sign"));
        assert!(!email_looks_valid("@example.com"));
        assert!(!email_looks_valid("reader@"));
        assert!(!email_looks_valid("reader@localhost"));
        assert!(!email_looks_valid("reader @example.com"));
    }
}
