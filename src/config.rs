use duration_str::{deserialize_duration, parse as duration_parse};
use serde::{de, Deserialize, Deserializer};
use std::time::Duration;

use crate::discover::Discover;
use crate::result::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub scrape_configs: Vec<ScrapeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub scrape_interval: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub scrape_timeout: Duration,
    #[serde(
        deserialize_with = "deserialize_duration",
        default = "default_evaluation_interval"
    )]
    pub evaluation_interval: Duration,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ScrapeConfig {
    pub job_name: String,
    #[serde(default, deserialize_with = "deserialize_option_duration")]
    pub scrape_interval: Option<Duration>,
    #[serde(default, deserialize_with = "deserialize_option_duration")]
    pub scrape_timeout: Option<Duration>,
    #[serde(default = "default_metrics_path")]
    pub metrics_path: String,
    #[serde(default = "default_scheme")]
    pub scheme: String,
    #[serde(flatten)]
    pub discover: Discover,
}

pub fn parse(config_file: &str) -> Result<Config, Error> {
    let config_yaml_content = std::fs::read(config_file)?;
    let mut config: Config = serde_yaml::from_slice(&config_yaml_content)?;
    for scrape_config in config.scrape_configs.iter_mut() {
        if scrape_config.scrape_timeout.is_none() {
            scrape_config.scrape_timeout = Some(config.global.scrape_timeout)
        }
    }
    Ok(config)
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

fn default_scheme() -> String {
    "http".to_string()
}

fn default_evaluation_interval() -> Duration {
    duration_parse("15s").unwrap()
}

pub fn deserialize_option_duration<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Some(duration) = Option::<String>::deserialize(deserializer)? {
        return Ok(Some(
            duration_str::parse(&duration).map_err(de::Error::custom)?,
        ));
    }
    Ok(None)
}
