//! Integration tests for subscriber one-time-token and edit subscription page.

#![recursion_limit = "512"]

use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use chrono::Utc;
use lariv_rs::app::App;
use lariv_rs::db::DbTag;
use lariv_rs::http::into_axum_router;
use lariv_rs::plugins::{filesystem, otp, signup, users::{self, UsersTag, auth, entities::user::Entity as UserEntity}, website};
use resummarized::publisher::entities::subscriber::{self, Entity as SubscriberEntity, NewsletterInterval};
use resummarized::{publisher, stock_markets, website_seed};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, Database, EntityTrait, QueryFilter, Statement};
use tower::util::ServiceExt;
use uuid::Uuid;

const STACK_SIZE: usize = 64 * 1024 * 1024;

fn temp_config(db_url: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "resummarized-edit-it-{}-{}.toml",
        std::process::id(),
        Uuid::new_v4()
    ));
    let body = format!(
        r#"database_url = "{db_url}"
[users]
adminEmail = "admin@test.local"
adminPassword = "adminadmin"
signingKey = "dGVzdC1zaWduaW5nLWtleS1wYWRkZWQtdG8tNjQtYnl0ZXMhISEhISEhISEhISE="
jwtIssuer = "cmVzdW1tYXJpemVkLXRlc3QtaXNzdWVyLXBhZGRlZC10by02NC1ieXRlcyE="
"#
    );
    std::fs::write(&path, body).expect("write temp config");
    path
}

#[test]
fn subscribe_edit_integration_flow() {
    std::thread::Builder::new()
        .name("subscribe-edit-test".into())
        .stack_size(STACK_SIZE)
        .spawn(|| {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();
            rt.block_on(async {
                let base_admin_url = std::env::var("DATABASE_ADMIN_URL")
                    .unwrap_or_else(|_| "postgres://postgres:0708@localhost:5432/postgres".into());
                let db_name = format!("test_sub_edit_{}", Uuid::new_v4().simple());

                // Create isolated test database
                let admin_conn = Database::connect(&base_admin_url)
                    .await
                    .expect("connect admin postgres");
                admin_conn
                    .execute(Statement::from_string(
                        admin_conn.get_database_backend(),
                        format!("CREATE DATABASE \"{db_name}\""),
                    ))
                    .await
                    .expect("create test database");

                let test_db_url = format!("postgres://postgres:0708@localhost:5432/{db_name}");

                let app = App::new_web_app();
                let app = users::install(app);
                let app = otp::install(app);
                let app = signup::install(app);
                let app = filesystem::install(app);
                let app = stock_markets::install(app);
                let app = publisher::install(app);
                let app = website::install(app);
                let app = website_seed::install(app);

                let path = temp_config(&test_db_url);
                let app = app.load_config(&path).await.expect("load_config");
                std::fs::remove_file(&path).ok();
                let app = app.mount();
                app.run_migrations().await.expect("run_migrations");
                app.run_seeds().await.expect("run_seeds");

                let users_state = app.get_capability_output::<UsersTag, _>();
                let db = app.get_capability_output::<DbTag, _>().conn.clone();
                let router = into_axum_router(&app);

                // 1. GET /subscribe/edit without params -> 400 Bad Request
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri("/subscribe/edit")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

                // 2. GET /subscribe/edit with non-existent email -> 404 Not Found
                let random_token = Uuid::new_v4();
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri(format!(
                                "/subscribe/edit?email=none@example.com&one_time_token={random_token}"
                            ))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::NOT_FOUND);

                // 3. Create a test subscriber in DB
                let initial_token = Uuid::new_v4();
                let sub = subscriber::ActiveModel {
                    id: Default::default(),
                    created_at: Set(Some(Utc::now())),
                    updated_at: Set(Some(Utc::now())),
                    email: Set("subscriber@example.com".into()),
                    subscription_date: Set(Utc::now()),
                    filter_exchanges: Set(None),
                    filter_entities: Set(None),
                    filter_event_types: Set(None),
                    newsletter_interval: Set(NewsletterInterval::Weekly),
                    one_time_token: Set(Some(initial_token)),
                }
                .insert(&db)
                .await
                .expect("insert subscriber");

                assert_eq!(sub.one_time_token, Some(initial_token));

                // 4. GET /subscribe/edit with mismatched token -> 403 Forbidden
                let mismatched_token = Uuid::new_v4();
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri(format!(
                                "/subscribe/edit?email=subscriber@example.com&one_time_token={mismatched_token}"
                            ))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::FORBIDDEN);

                // 5. GET /subscribe/edit with valid token
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri(format!(
                                "/subscribe/edit?email=subscriber@example.com&one_time_token={initial_token}"
                            ))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");

                assert_eq!(resp.status(), StatusCode::OK);
                // Check no-cache headers
                let cc = resp
                    .headers()
                    .get("cache-control")
                    .expect("cache-control header")
                    .to_str()
                    .unwrap();
                assert!(cc.contains("no-store") && cc.contains("no-cache") && cc.contains("must-revalidate"));

                // Read response body
                let body_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
                let body_str = String::from_utf8_lossy(&body_bytes);

                // Check DB to verify token was rotated
                let updated_sub = SubscriberEntity::find_by_id(sub.id)
                    .one(&db)
                    .await
                    .expect("query sub")
                    .expect("sub exists");
                let second_token = updated_sub.one_time_token.expect("one_time_token is set");
                assert_ne!(second_token, initial_token);

                // The rendered HTML must contain the new rotated token in the form!
                assert!(
                    body_str.contains(&second_token.to_string()),
                    "HTML must contain the new rotated token"
                );
                assert!(body_str.contains("Cancel subscription"));
                assert!(body_str.contains("formaction=\"/subscribe/cancel\""));
                assert!(body_str.contains("confirm("));

                // 6. Old token is now invalid -> 403 Forbidden
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri(format!(
                                "/subscribe/edit?email=subscriber@example.com&one_time_token={initial_token}"
                            ))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::FORBIDDEN);

                // 7. POST /subscribe/edit with old token -> 403 Forbidden
                let csrf_tok = lariv_rs::html_form::generate_csrf_token();
                let post_body_invalid = format!(
                    "csrf_token={csrf_tok}&email=subscriber@example.com&one_time_token={initial_token}&newsletter_interval=daily"
                );
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/subscribe/edit")
                            .header("content-type", "application/x-www-form-urlencoded")
                            .header("cookie", format!("csrf_token={csrf_tok}"))
                            .body(Body::from(post_body_invalid))
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::FORBIDDEN);

                // 8. POST /subscribe/edit with valid second_token -> updates preferences & rotates token
                let post_body_valid = format!(
                    "csrf_token={csrf_tok}&email=subscriber@example.com&one_time_token={second_token}&newsletter_interval=daily&filter_exchanges=nse"
                );
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/subscribe/edit")
                            .header("content-type", "application/x-www-form-urlencoded")
                            .header("cookie", format!("csrf_token={csrf_tok}"))
                            .body(Body::from(post_body_valid))
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::OK);

                let cc = resp
                    .headers()
                    .get("cache-control")
                    .expect("cache-control header")
                    .to_str()
                    .unwrap();
                assert!(cc.contains("no-store") && cc.contains("no-cache"));

                let post_body_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
                let post_body_str = String::from_utf8_lossy(&post_body_bytes);
                assert!(post_body_str.contains("Your subscription preferences have been updated"));

                // Verify DB was updated
                let final_sub = SubscriberEntity::find_by_id(sub.id)
                    .one(&db)
                    .await
                    .expect("query sub")
                    .expect("sub exists");
                assert_eq!(final_sub.newsletter_interval, NewsletterInterval::Daily);
                let third_token = final_sub.one_time_token.expect("third_token is set");
                assert_ne!(third_token, second_token);
                assert_ne!(third_token, initial_token);

                // 9. Verify Rate Limiter:
                // We have used 2 link generations so far (1 in GET, 1 in POST).
                // Let's do 3 more GETs to reach 5 total generations:
                let mut current_token = third_token;
                for _ in 0..3 {
                    let resp = router
                        .clone()
                        .oneshot(
                            Request::builder()
                                .method("GET")
                                .uri(format!(
                                    "/subscribe/edit?email=subscriber@example.com&one_time_token={current_token}"
                                ))
                                .body(Body::empty())
                                .unwrap(),
                        )
                        .await
                        .expect("response");
                    assert_eq!(resp.status(), StatusCode::OK);
                    let sub = SubscriberEntity::find_by_id(sub.id)
                        .one(&db)
                        .await
                        .unwrap()
                        .unwrap();
                    current_token = sub.one_time_token.expect("current_token is set");
                }

                // Total link generations for subscriber@example.com is now 5 within 5 minutes.
                // The 6th request must trigger 429 Too Many Requests!
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri(format!(
                                "/subscribe/edit?email=subscriber@example.com&one_time_token={current_token}"
                            ))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

                // 10. Authenticate admin user
                let admin = UserEntity::find()
                    .filter(lariv_rs::plugins::users::entities::user::Column::Email.eq("admin@test.local"))
                    .one(&db)
                    .await
                    .expect("find admin")
                    .expect("admin exists");
                let admin_token = auth::login_token(
                    &admin,
                    &users_state.signing_key,
                    &users_state.jwt_issuer,
                )
                .expect("admin login token");

                // 11. Admin Detail Page Action: GET /publisher/subscribers/{id}/send-edit-link modal
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri(format!("/publisher/subscribers/{}/send-edit-link", sub.id))
                            .header("cookie", format!("auth-token={admin_token}"))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::OK);
                let modal_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
                let modal_str = String::from_utf8_lossy(&modal_bytes);
                assert!(modal_str.contains("Send Subscription Change Email"));
                assert!(modal_str.contains("subscriber@example.com"));

                // 12. Admin Detail Page Action: POST /publisher/subscribers/{id}/send-edit-link
                let token_before_admin = current_token;
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri(format!("/publisher/subscribers/{}/send-edit-link", sub.id))
                            .header("cookie", format!("auth-token={admin_token}"))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::SEE_OTHER);
                assert_eq!(
                    resp.headers().get("location").unwrap().to_str().unwrap(),
                    format!("/publisher/subscribers/{}/", sub.id)
                );

                // Check that subscriber's token was rotated
                let sub_after_send = SubscriberEntity::find_by_id(sub.id)
                    .one(&db)
                    .await
                    .unwrap()
                    .unwrap();
                let token_after_admin = sub_after_send.one_time_token.expect("token is set");
                assert_ne!(token_after_admin, token_before_admin);

                // 13. Create a second subscriber for Bulk Action test
                let sub2_token = Uuid::new_v4();
                let sub2 = subscriber::ActiveModel {
                    id: Default::default(),
                    created_at: Set(Some(Utc::now())),
                    updated_at: Set(Some(Utc::now())),
                    email: Set("subscriber2@example.com".into()),
                    subscription_date: Set(Utc::now()),
                    filter_exchanges: Set(None),
                    filter_entities: Set(None),
                    filter_event_types: Set(None),
                    newsletter_interval: Set(NewsletterInterval::Daily),
                    one_time_token: Set(Some(sub2_token)),
                }
                .insert(&db)
                .await
                .expect("insert subscriber 2");

                // Bulk modal GET
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri(format!(
                                "/publisher/subscribers/bulk-send-edit-link?ids={},{}",
                                sub.id, sub2.id
                            ))
                            .header("cookie", format!("auth-token={admin_token}"))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::OK);
                let bulk_modal_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
                let bulk_modal_str = String::from_utf8_lossy(&bulk_modal_bytes);
                assert!(bulk_modal_str.contains("Send Subscription Change Emails"));
                assert!(bulk_modal_str.contains("selected subscriber(s)"));

                // Bulk action POST
                let csrf_tok = lariv_rs::html_form::generate_csrf_token();
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/publisher/subscribers/bulk-send-edit-link")
                            .header("content-type", "application/x-www-form-urlencoded")
                            .header("cookie", format!("auth-token={admin_token}; csrf_token={csrf_tok}"))
                            .body(Body::from(format!("csrf_token={csrf_tok}&ids={},{}", sub.id, sub2.id)))
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::SEE_OTHER);
                assert_eq!(
                    resp.headers().get("location").unwrap().to_str().unwrap(),
                    "/publisher/subscribers/"
                );

                // Verify both subscribers had tokens rotated
                let sub1_final = SubscriberEntity::find_by_id(sub.id)
                    .one(&db)
                    .await
                    .unwrap()
                    .unwrap();
                let sub2_final = SubscriberEntity::find_by_id(sub2.id)
                    .one(&db)
                    .await
                    .unwrap()
                    .unwrap();
                assert_ne!(sub1_final.one_time_token.unwrap(), token_after_admin);
                assert_ne!(sub2_final.one_time_token.unwrap(), sub2_token);

                // 14. Preferences: test saving and loading email templates
                let mut prefs = resummarized::publisher::preferences::load_preferences(&db)
                    .await
                    .expect("load preferences");
                prefs.edit_link_email_subject = "Custom Subj: {{ email }}".into();
                prefs.edit_link_email_template = "<p>Custom Body: {{ edit_link }}</p>".into();
                prefs.edit_opened_email_subject = "Opened Subj".into();
                prefs.edit_opened_email_template = "<p>Opened Body</p>".into();
                prefs.edit_updated_email_subject = "Updated Subj".into();
                prefs.edit_updated_email_template = "<p>Updated Body</p>".into();
                resummarized::publisher::preferences::save_preferences(&db, prefs)
                    .await
                    .expect("save preferences");

                let reloaded_prefs = resummarized::publisher::preferences::load_preferences(&db)
                    .await
                    .expect("reload preferences");
                assert_eq!(reloaded_prefs.edit_link_email_subject, "Custom Subj: {{ email }}");
                assert_eq!(
                    reloaded_prefs.edit_link_email_template,
                    "<p>Custom Body: {{ edit_link }}</p>"
                );
                assert_eq!(reloaded_prefs.edit_opened_email_subject, "Opened Subj");
                assert_eq!(reloaded_prefs.edit_opened_email_template, "<p>Opened Body</p>");
                assert_eq!(reloaded_prefs.edit_updated_email_subject, "Updated Subj");
                assert_eq!(reloaded_prefs.edit_updated_email_template, "<p>Updated Body</p>");

                // 15. Cancel Subscription:
                // Attempt with invalid token -> 403 Forbidden
                let csrf_tok = lariv_rs::html_form::generate_csrf_token();
                let bad_token = Uuid::new_v4();
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/subscribe/cancel")
                            .header("content-type", "application/x-www-form-urlencoded")
                            .header("cookie", format!("csrf_token={csrf_tok}"))
                            .body(Body::from(format!(
                                "csrf_token={csrf_tok}&email=subscriber2@example.com&one_time_token={bad_token}"
                            )))
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::FORBIDDEN);

                // Attempt with valid token for sub2 -> deletes sub2
                let sub2_current_token = sub2_final.one_time_token.unwrap();
                let resp = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/subscribe/cancel")
                            .header("content-type", "application/x-www-form-urlencoded")
                            .header("cookie", format!("csrf_token={csrf_tok}"))
                            .body(Body::from(format!(
                                "csrf_token={csrf_tok}&email=subscriber2@example.com&one_time_token={sub2_current_token}"
                            )))
                            .unwrap(),
                    )
                    .await
                    .expect("response");
                assert_eq!(resp.status(), StatusCode::OK);
                let cancel_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
                let cancel_str = String::from_utf8_lossy(&cancel_bytes);
                assert!(cancel_str.contains("Subscription Cancelled"));
                assert!(cancel_str.contains("Your subscription has been successfully cancelled"));

                // Verify sub2 was deleted from DB
                let sub2_after_cancel = SubscriberEntity::find_by_id(sub2.id)
                    .one(&db)
                    .await
                    .expect("query sub2");
                assert!(sub2_after_cancel.is_none(), "sub2 should be deleted from DB");

                // Cleanup test database
                drop(db);
                admin_conn
                    .execute(Statement::from_string(
                        admin_conn.get_database_backend(),
                        format!("DROP DATABASE \"{db_name}\" WITH (FORCE)"),
                    ))
                    .await
                    .expect("drop test database");
            });
        })
        .expect("spawn subscribe-edit-test thread")
        .join()
        .expect("subscribe-edit-test thread");
}
