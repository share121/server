pub mod config;
pub mod entry;
pub mod invert;
pub mod puller;
pub mod send_err;
pub mod unique_path;

use crate::{config::DownloadConfig, entry::DownloadEntry};
use aria2_gid::Gid;
use dashmap::DashMap;
use spin::mutex::SpinMutex;
use std::sync::Arc;
use url::Url;

pub struct Downloader {
    list: Arc<DashMap<Gid, (DownloadEntry, AddOptions)>>,
    parallelism: usize,
    pub config: Arc<SpinMutex<DownloadConfig>>,
}

impl Downloader {
    pub fn new(config: DownloadConfig) -> Self {
        Self::with_capacity(config, 0)
    }
    pub fn with_capacity(config: DownloadConfig, capacity: usize) -> Self {
        Self {
            list: Arc::new(DashMap::with_capacity(capacity)),
            parallelism: 0,
            tasks: Vec::with_capacity(capacity),
            config: Arc::new(SpinMutex::new(config)),
        }
    }
    pub fn set_parallelism(&mut self, parallelism: usize) {
        self.parallelism = parallelism;
        self.run();
    }
    pub fn run(&mut self) {}
    pub fn add(&mut self, options: AddOptions) -> Result<Gid, reqwest::Error> {
        let call_dbg = format!("downloader.add({options:?})");
        log::debug!("{call_dbg}");
        let gid = loop {
            let temp = Gid::new();
            if !self.list.contains_key(&temp) {
                break temp;
            }
            log::debug!("{call_dbg}: Gid collision, retrying");
        };
        log::debug!("{call_dbg}: Assigned Gid {gid}");
        let entry = DownloadEntry::new(
            gid,
            options.url.clone(),
            options.config.clone(),
            self.config.clone(),
        );
        log::debug!("{call_dbg}: Inserted entry {gid}: {entry:?}");
        self.list.insert(gid, (entry, options));
        self.run();
        Ok(gid)
    }
    pub fn remove(&mut self, gid: Gid) -> Option<(Gid, (DownloadEntry, AddOptions))> {
        log::debug!("downloader.remove({gid})");
        let mut entry = self.list.remove(&gid);
        log::debug!("downloader.remove({gid}): {entry:?}");
        if let Some((_, (entry, _))) = &mut entry {
            log::debug!("downloader.remove({gid}): Aborting");
            entry.abort();
            log::debug!("downloader.remove({gid}): Aborted");
        }
        self.run();
        entry
    }
    pub fn stop(&mut self, gid: Gid) {
        log::debug!("downloader.stop({gid})");
        {
            let mut entry = self.list.get_mut(&gid);
            log::debug!("downloader.stop({gid}): {entry:?}");
            if let Some(entry) = &mut entry {
                log::debug!("downloader.stop({gid}): Aborting");
                entry.0.abort();
                log::debug!("downloader.stop({gid}): Aborted");
            }
        }
        self.run();
    }
    pub fn running_count(&self) -> usize {
        self.list
            .iter()
            .map(|e| if e.0.is_running() { 1 } else { 0 })
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct AddOptions {
    pub url: Url,
    pub immediate_download: bool,
    pub config: DownloadConfig,
}
