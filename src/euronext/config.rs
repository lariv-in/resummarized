use serde::Deserialize;

use lariv_rs::config::ConfigSection;

pub struct EuronextConfigTag;

impl ConfigSection for EuronextConfigTag {
    const KEY: Option<&'static str> = Some("euronext");
}

#[derive(Debug, Clone, Deserialize)]
pub struct EuronextConfig {
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval_secs() -> u64 {
    300
}

impl Default for EuronextConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval_secs(),
        }
    }
}
