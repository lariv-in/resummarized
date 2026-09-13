use std::time::Duration;

use lariv_rs::{app::MountedApp, hooks::RunServeStartup, traits::get::GetByTag};
use tracing::warn;

use super::{EdgarTag, fetch, state::EdgarState};

#[derive(Clone, Copy, Default)]
pub struct ServeStartupHook;

#[async_trait::async_trait]
impl<M, Idx> RunServeStartup<M, Idx> for ServeStartupHook
where
    M: GetByTag<EdgarTag, Idx, Value = EdgarState> + Sync,
{
    async fn run_serve_startup(app: &MountedApp<M>) -> anyhow::Result<()> {
        let state = app.get_capability_output::<EdgarTag, Idx>().clone();
        spawn_poller(state);
        Ok(())
    }
}

pub fn spawn_poller(state: EdgarState) {
    tokio::spawn(async move {
        run_poll_loop(state).await;
    });
}

async fn run_poll_loop(state: EdgarState) {
    let interval = Duration::from_secs(state.config.poll_interval_secs.max(60));
    loop {
        if let Err(e) = fetch::fetch_all(&state).await {
            warn!(error = %e, "EDGAR poll cycle failed");
        }
        tokio::time::sleep(interval).await;
    }
}
