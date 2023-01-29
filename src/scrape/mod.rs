use log::debug;

use self::job::Job;
use crate::config::{GlobalConfig, ScrapeConfig};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, SendError, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

mod job;
mod scraper;

#[derive(Debug)]
pub struct Manager {
    tx: Sender<Message>,
    jobs: Arc<Mutex<HashMap<String, Job>>>,
}

pub enum Message {
    Reload {
        scrape_configs: Vec<ScrapeConfig>,
        default_interval: Duration,
        default_timeout: Duration,
    },
}

impl Manager {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let rt = Self {
            tx,
            jobs: Arc::new(Mutex::new(HashMap::default())),
        };
        rt.start(rx);
        rt
    }
    fn start(&self, rx: Receiver<Message>) -> JoinHandle<()> {
        let jobs_lock = self.jobs.clone();
        thread::Builder::new()
            .name("ScrapeManager".to_string())
            .spawn(move || loop {
                match rx.recv().unwrap() {
                    Message::Reload {
                        default_interval,
                        default_timeout,
                        scrape_configs,
                    } => {
                        let mut jobs = jobs_lock.lock().unwrap();
                        for (_, job) in jobs.iter_mut() {
                            job.stop();
                        }
                        jobs.clear();
                        for config in scrape_configs {
                            let mut new_job = Job::new(
                                &config.job_name,
                                config.clone(),
                                default_interval,
                                default_timeout,
                            )
                            .unwrap();
                            new_job.start().unwrap();
                            jobs.insert(new_job.name.clone(), new_job);
                        }
                    }
                }
            })
            .unwrap()
    }
    pub fn reload(
        &self,
        global_config: &GlobalConfig,
        scrape_configs: &[ScrapeConfig],
    ) -> Result<(), SendError<Message>> {
        debug!("reload Scrape Manager");
        self.tx.send(Message::Reload {
            scrape_configs: scrape_configs.to_vec(),
            default_interval: global_config.scrape_interval,
            default_timeout: global_config.scrape_timeout,
        })
    }
}
