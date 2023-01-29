use std::process::exit;

use env_logger::{init_from_env, Env};
use log::{debug, error};
use signal_hook::consts::{SIGINT, SIGKILL, SIGTERM, TERM_SIGNALS};
use signal_hook::iterator::SignalsInfo;
use signal_hook::{consts::SIGHUP, iterator::exfiltrator::WithOrigin};
use {cli::parse as cli_parse, config::parse as config_parse, scrape::Manager as ScrapeManager};

mod cli;
mod config;
mod discover;
mod result;
mod scrape;

fn main() {
    let flags = cli_parse();
    init_from_env(Env::new().default_filter_or(flags.log_level.as_str()));
    debug!("opts: {:?}", &flags);
    let config = config_parse(&flags.config_file).unwrap();
    debug!("config: {:?}", &config);
    let scrape_manager = ScrapeManager::new();
    scrape_manager
        .reload(&config.global, &config.scrape_configs)
        .unwrap();
    let mut signals = vec![SIGHUP];
    signals.extend(TERM_SIGNALS);
    let mut signals = SignalsInfo::<WithOrigin>::new(signals).unwrap();
    for info in &mut signals {
        debug!("Received a signal {:?}", info.signal);
        match info.signal {
            SIGINT | SIGTERM | SIGKILL => exit(0),
            SIGHUP => {
                let new_config = config_parse(&flags.config_file).unwrap();
                scrape_manager
                    .reload(&new_config.global, &new_config.scrape_configs)
                    .unwrap();
            }
            _ => {
                error!("unsupport signal: {:?}", info);
            }
        }
    }
}
