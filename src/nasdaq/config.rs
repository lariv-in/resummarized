use serde::Deserialize;

use lariv_rs::config::ConfigSection;

pub struct NasdaqConfigTag;

impl ConfigSection for NasdaqConfigTag {
    const KEY: Option<&'static str> = Some("nasdaq");
}

#[derive(Debug, Clone, Deserialize)]
pub struct NasdaqConfig {
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval_secs() -> u64 {
    300
}

impl Default for NasdaqConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval_secs(),
        }
    }
}
