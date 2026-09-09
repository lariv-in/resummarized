use serde::Deserialize;

use lariv_rs::config::ConfigSection;

pub struct BseConfigTag;

impl ConfigSection for BseConfigTag {
    const KEY: Option<&'static str> = Some("bse");
}

#[derive(Debug, Clone, Deserialize)]
pub struct BseConfig {
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval_secs() -> u64 {
    300
}

impl Default for BseConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval_secs(),
        }
    }
}
