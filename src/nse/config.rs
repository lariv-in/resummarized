use serde::Deserialize;

use lariv_rs::config::ConfigSection;

pub struct NseConfigTag;

impl ConfigSection for NseConfigTag {
    const KEY: Option<&'static str> = Some("nse");
}

#[derive(Debug, Clone, Deserialize)]
pub struct NseConfig {
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval_secs() -> u64 {
    300
}

impl Default for NseConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval_secs(),
        }
    }
}
