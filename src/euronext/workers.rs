use std::time::Duration;

use lariv_rs::{app::MountedApp, hooks::RunServeStartup, traits::get::GetByTag};
use tracing::warn;

use super::{EuronextTag, fetch, state::EuronextState};

#[derive(Clone, Copy, Default)]
pub struct ServeStartupHook;

#[async_trait::async_trait]
impl<M, Idx> RunServeStartup<M, Idx> for ServeStartupHook
where
    M: GetByTag<EuronextTag, Idx, Value = EuronextState> + Sync,
{
    async fn run_serve_startup(app: &MountedApp<M>) -> anyhow::Result<()> {
        let state = app.get_capability_output::<EuronextTag, Idx>().clone();
        spawn_poller(state);
        Ok(())
    }
}

pub fn spawn_poller(state: EuronextState) {
    tokio::spawn(async move {
        run_poll_loop(state).await;
    });
}

async fn run_poll_loop(state: EuronextState) {
    let interval = Duration::from_secs(state.config.poll_interval_secs.max(60));
    loop {
        if let Err(e) = fetch::fetch_all(&state).await {
            warn!(error = %e, "Euronext poll cycle failed");
        }
        tokio::time::sleep(interval).await;
    }
}
