use serde::Deserialize;

use lariv_rs::config::ConfigSection;

pub struct JpxConfigTag;

impl ConfigSection for JpxConfigTag {
    const KEY: Option<&'static str> = Some("jpx");
}

#[derive(Debug, Clone, Deserialize)]
pub struct JpxConfig {
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval_secs() -> u64 {
    300
}

impl Default for JpxConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval_secs(),
        }
    }
}
