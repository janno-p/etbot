use config::{Config, ConfigError, File};
use duration_str::deserialize_duration;
use serde::Deserialize;
use std::{env, time::Duration};

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Database {
    pub filename: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Discord {
    pub token: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct PotatoFeeder {
    #[serde(alias = "channel-id")]
    pub channel_id: u64,
    #[serde(deserialize_with = "deserialize_duration")]
    interval: Duration,
    amount: u32,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub discord: Discord,
    #[serde(alias = "potato-feeder")]
    pub potato_feeder: PotatoFeeder,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "production".into());

        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .build()?;

        s.try_deserialize()
    }
}
