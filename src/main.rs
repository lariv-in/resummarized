#![recursion_limit = "512"]

use lariv_rs::app::App;
use lariv_rs::plugins::{dashboard, filesystem, llm_assistant, otp, signup, users, website};
use resummarized::{bse, nse, publisher, website_seed};
use tracing_subscriber::EnvFilter;

#[lariv_rs::main(
    stack_size = 64 * 1024 * 1024,
    flavor = "multi_thread",
    thread_name = "resummarized-server"
)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse().expect("directive"))
                .add_directive("llm_assistant::imap=info".parse().expect("directive")),
        )
        .init();

    let app = App::new_web_app();
    let app = users::install(app);
    let app = otp::install(app);
    let app = signup::install(app);
    let app = filesystem::install(app);
    let app = llm_assistant::install(app);
    let app = nse::install(app);
    let app = bse::install(app);
    let app = publisher::install(app);
    let app = dashboard::install(app);
    // After dashboard so website can own `/` (CMS home) over the auth redirect.
    let app = website::install(app);
    let app = website_seed::install(app);

    let app = app.load_config("config.toml").await?;
    let app = app.mount();
    app.run_migrations().await?;
    app.run().await?;
    Ok(())
}
