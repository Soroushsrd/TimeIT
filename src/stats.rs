use std::{collections::HashMap, path::PathBuf, time::SystemTime};
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct TimeEntry {
    pub path: PathBuf,
    pub language: String,
    pub project: Option<String>,
    pub duration: Duration,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
}

#[derive(Debug, Clone)]
pub struct DailyStats {
    pub date: String, //yy-mm-dd format
    pub total_time: Duration,
    // TODO: rewrite this module as a binary tree?
    pub entries_by_lang: HashMap<String, Duration>,
    pub entries_by_project: HashMap<String, Duration>,
    pub entries_by_file: HashMap<PathBuf, Duration>,
}

impl DailyStats {
    pub fn new(date: String) -> Self {
        Self {
            date,
            total_time: Duration::ZERO,
            entries_by_lang: HashMap::new(),
            entries_by_project: HashMap::new(),
            entries_by_file: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, entry: &TimeEntry) {
        self.total_time += entry.duration;

        *self
            .entries_by_lang
            .entry(entry.language.clone())
            .or_insert(Duration::ZERO) += entry.duration;
        *self
            .entries_by_file
            .entry(entry.path.clone())
            .or_insert(Duration::ZERO) += entry.duration;

        if let Some(project) = &entry.project {
            *self
                .entries_by_project
                .entry(project.clone())
                .or_insert(Duration::ZERO) += entry.duration;
        }
    }
}
