use super::scraper::Scraper;
use crate::{config::ScrapeConfig, result::Error};
use std::{
    ops::DerefMut,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{Builder, JoinHandle},
    time::Duration,
};

#[derive(Debug)]
pub struct Job {
    pub name: String,
    default_interval: Duration,
    default_timeout: Duration,
    scrape_config: ScrapeConfig,
    scrapers: Arc<Mutex<Vec<Scraper>>>,
    stop_rx: Option<Sender<()>>,
}

impl Job {
    pub fn new(
        job_name: &str,
        scrape_config: ScrapeConfig,
        default_interval: Duration,
        default_timeout: Duration,
    ) -> Result<Self, Error> {
        Ok(Self {
            default_interval,
            default_timeout,
            scrape_config,
            name: job_name.to_string(),
            scrapers: Arc::new(Mutex::new(Vec::default())),
            stop_rx: Option::default(),
        })
    }
    pub fn start(&mut self) -> Result<JoinHandle<()>, std::io::Error> {
        let (discover_tx, discover_rx) = channel();
        let (stop_tx, stop_rx) = channel();
        self.stop_rx = Some(stop_tx);
        self.scrape_config.discover.start(discover_tx);
        let scrapers_lock = self.scrapers.clone();
        let config = self.scrape_config.clone();
        let mut scrape_interval = self.default_interval;
        if let Some(interval) = &self.scrape_config.scrape_interval {
            scrape_interval = interval.to_owned();
        }
        let mut scrape_timeout = self.default_timeout;
        if let Some(timeout) = &self.scrape_config.scrape_timeout {
            scrape_timeout = timeout.to_owned();
        }
        Builder::new()
            .name(format!("job: {}", self.scrape_config.job_name))
            .spawn(move || loop {
                if let Ok(()) = stop_rx.try_recv() {
                    let mut scrapers = scrapers_lock.lock().unwrap();
                    for scraper in scrapers.iter_mut() {
                        scraper.stop();
                    }
                    return;
                }
                if let Ok(targets) = discover_rx.recv_timeout(Duration::from_secs(1)) {
                    let mut new_scrapers = Vec::default();

                    for target_group in &targets {
                        for target in &target_group.targets {
                            let mut new_scraper = Scraper::new(
                                format!("{}://{}{}", config.scheme, target, config.metrics_path),
                                scrape_interval,
                                scrape_timeout,
                            );
                            new_scraper.start();
                            new_scrapers.push(new_scraper);
                        }
                    }
                    let mut scraper_guard = scrapers_lock.lock().unwrap();
                    let old_scrapers = scraper_guard.deref_mut();
                    for old_scraper in old_scrapers.iter() {
                        old_scraper.stop();
                    }
                    old_scrapers.clear();
                    for new_scraper in new_scrapers {
                        old_scrapers.push(new_scraper);
                    }
                }
            })
    }

    pub fn stop(&self) {
        self.scrape_config.discover.stop();
        if let Some(stop_tx) = &self.stop_rx {
            stop_tx.send(()).unwrap();
        }
    }
}
