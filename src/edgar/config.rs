use serde::Deserialize;

use lariv_rs::config::ConfigSection;

pub struct EdgarConfigTag;

impl ConfigSection for EdgarConfigTag {
    const KEY: Option<&'static str> = Some("edgar");
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdgarConfig {
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval_secs() -> u64 {
    300
}

impl Default for EdgarConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval_secs(),
        }
    }
}
