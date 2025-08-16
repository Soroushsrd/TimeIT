use std::{path::PathBuf, time::SystemTime};

use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FileSession {
    pub path: PathBuf,
    pub language: String,
    pub project: Option<String>,
    pub total_duration: Duration,
    pub current_session_start: Option<Instant>,
    pub is_active: bool,
    pub last_activity: SystemTime,
}

impl FileSession {
    pub fn new(path: PathBuf, language: String) -> Self {
        Self {
            language,
            project: detect_project(&path),
            path,
            total_duration: Duration::ZERO,
            current_session_start: Some(Instant::now()),
            is_active: true,
            last_activity: SystemTime::now(),
        }
    }

    pub fn pause(&mut self) {
        if self.is_active {
            self.is_active = false;
            let start = self.last_activity.elapsed().unwrap_or(Duration::ZERO);
            self.total_duration += start;
        }
    }

    pub fn resume(&mut self) {
        if !self.is_active {
            self.current_session_start = Some(Instant::now());
            self.is_active = true;
            self.last_activity = SystemTime::now();
        }
    }

    pub fn get_current_duration(&self) -> Duration {
        let start_time = if let Some(start) = self.current_session_start {
            start.elapsed()
        } else {
            Duration::ZERO
        };

        self.total_duration + start_time
    }
}

pub fn detect_project(path: &PathBuf) -> Option<String> {
    let mut current_dir = path.as_path();

    loop {
        if let Some(parent) = current_dir.parent() {
            if parent.join(".git").exists()
                || parent.join("Cargo.toml").exists()
                || parent.join("package.json").exists()
                || parent.join("pyproject.toml").exists()
                || parent.join("requirements.txt").exists()
                || parent.join("go.mod").exists()
                || parent.join("pom.xml").exists()
            {
                return parent
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|name| name.to_string());
            }
            current_dir = parent;
        } else {
            break;
        }
    }
    None
}
