//! Compile smoke test for the resummarized plugin stack.

#![recursion_limit = "512"]

use std::path::PathBuf;

use lariv_rs::app::App;
use lariv_rs::plugins::{dashboard, filesystem, llm_assistant, otp, signup, users, website};
use resummarized::{bse, nse, website_seed};

const STACK_SIZE: usize = 64 * 1024 * 1024;

const MINIMAL_DB_TOML: &str = r#"database_url = "sqlite::memory:"
[users]
adminEmail = "admin@test.local"
adminPassword = "adminadmin"
"#;

fn temp_config(body: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "resummarized-mount-{}-{}.toml",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::write(&path, body).expect("write temp config");
    path
}

#[test]
fn resummarized_stack_mounts() {
    std::thread::Builder::new()
        .name("resummarized-mount".into())
        .stack_size(STACK_SIZE)
        .spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(async {
                let app = App::new_web_app();
                let app = users::install(app);
                let app = otp::install(app);
                let app = signup::install(app);
                let app = filesystem::install(app);
                let app = llm_assistant::install(app);
                let app = nse::install(app);
                let app = bse::install(app);
                let app = dashboard::install(app);
                let app = website::install(app);
                let app = website_seed::install(app);

                let path = temp_config(MINIMAL_DB_TOML);
                let app = app.load_config(&path).await.expect("load_config");
                std::fs::remove_file(&path).ok();
                let _mounted = app.mount();
            });
        })
        .expect("spawn resummarized-mount thread")
        .join()
        .expect("resummarized-mount thread");
}
