use log::debug;
use reqwest::blocking::ClientBuilder;
use std::{
    sync::mpsc::{channel, Sender},
    thread::{self, Builder},
    time::Duration,
};

#[derive(Debug)]
pub struct Scraper {
    url: String,
    interval: Duration,
    timeout: Duration,
    stop_tx: Option<Sender<()>>,
}

impl Scraper {
    pub fn new(url: String, interval: Duration, timeout: Duration) -> Self {
        Self {
            url,
            interval,
            timeout,
            stop_tx: Option::default(),
        }
    }

    pub fn start(&mut self) {
        let url = self.url.clone();
        let (stop_tx, stop_rx) = channel();
        self.stop_tx = Some(stop_tx);
        let interval = self.interval;
        let timeout = self.timeout;
        Builder::new()
            .name(format!("scraper: {}", &url))
            .spawn(move || loop {
                thread::sleep(interval);
                if let Ok(()) = stop_rx.try_recv() {
                    return;
                }
                let scrape_result = ClientBuilder::new()
                    .timeout(timeout)
                    .build()
                    .unwrap()
                    .get(&url)
                    .send()
                    .unwrap();
                let status_code = scrape_result.status();
                debug!("status_code: {:?}", status_code);
                if !status_code.is_success() {
                    continue;
                }
            })
            .unwrap();
    }

    pub fn stop(&self) {
        if let Some(stop_tx) = &self.stop_tx {
            stop_tx.send(()).unwrap();
        }
    }
}
