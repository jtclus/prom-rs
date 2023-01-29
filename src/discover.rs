use serde::Deserialize;
use std::sync::mpsc::Sender;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Discover {
    pub static_configs: Option<Targets>,
}

pub type Targets = Vec<Group>;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    pub targets: Vec<String>,
}

impl Discover {
    pub fn start(&self, tx: Sender<Targets>) {
        if let Some(static_configs) = &self.static_configs {
            tx.send(static_configs.clone()).unwrap();
        }
    }
    pub fn stop(&self) {
        if self.static_configs.is_some() {}
    }
}
