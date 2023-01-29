use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "prom-server")]
#[command(version = "0.1-dev")]
pub struct Opts {
    #[arg(long = "web.listen-address", default_value = "0.0.0.0:9090")]
    web_listen_address: String,
    #[arg(value_enum, long = "log.level", default_value = "error")]
    pub log_level: log::LevelFilter,
    #[arg(long = "config.file", short = 'c', default_value = "prometheus.yml")]
    pub config_file: String,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
    Info,
    Debug,
    Error,
}

pub fn parse() -> Opts {
    Opts::parse()
}
