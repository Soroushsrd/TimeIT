use anyhow::{Context, Result};
use std::{
    sync::{Arc, RwLock, mpsc},
    thread,
    time::{Duration, SystemTime},
};
use tracing::error;

// input watcher should look out for keyboard and mouse inputs
// it should filter for "any" event captured by rdev. it must also
// keep track of activity time and idle time and
//

#[derive(Debug, Clone)]
pub enum ActivityEvent {
    KeyboardActivity { time_stamp: SystemTime },
    MouseActivity { time_stamp: SystemTime },
    IdleDetected { duration: Duration },
    ActivityResumed,
}

/// keeps track of activities
#[derive(Debug, Clone)]
pub struct ActivityState {
    last_activity: Option<SystemTime>,
    pub is_idle: bool,
}

impl ActivityState {
    pub fn new() -> Self {
        Self {
            last_activity: None,
            is_idle: false,
        }
    }

    pub fn is_recently_active(&self, within: Duration) -> bool {
        if let Some(activity) = self.last_activity {
            if let Ok(elapsed) = activity.elapsed() {
                return elapsed <= within;
            }
        }
        false
    }

    pub fn time_since_last_activity(&self) -> Option<Duration> {
        self.last_activity.and_then(|time| time.elapsed().ok())
    }
}

// Tracks input activity
// for now it considers all activities as valid
// but we should add threshold for mouse movements
// and other validation algos
#[derive(Debug, Clone)]
pub struct InputMonitor {
    state: Arc<RwLock<ActivityState>>,
    event_sender: mpsc::Sender<ActivityEvent>,
    idle_threshold: Duration,
}

impl InputMonitor {
    pub fn new(idle_threshold: Duration, event_sender: mpsc::Sender<ActivityEvent>) -> Self {
        Self {
            state: Arc::new(RwLock::new(ActivityState::new())),
            event_sender,
            idle_threshold,
        }
    }

    pub fn get_state(&self) -> Option<ActivityState> {
        match self.state.read() {
            Ok(event) => Some(event.clone()),
            Err(e) => {
                error!("Failed to get activity state: {e}");
                None
            }
        }
    }

    pub fn handle_keyboard_event(&mut self) -> Result<()> {
        let now = SystemTime::now();
        let mut state = self
            .state
            .write()
            .expect("failed to get a read lock on state");
        state.last_activity = Some(now);

        if state.is_idle {
            state.is_idle = false;
            self.event_sender
                .send(ActivityEvent::ActivityResumed)
                .context("failed to send activity resumed through channel")?;
        }

        self.event_sender
            .send(ActivityEvent::KeyboardActivity { time_stamp: now })
            .context("failed to send keyboard event")?;
        Ok(())
    }

    pub fn handle_mouse_event(&mut self) -> Result<()> {
        let now = SystemTime::now();
        let mut state = self
            .state
            .write()
            .expect("failed to get a read lock on state");
        state.last_activity = Some(now);

        if state.is_idle {
            state.is_idle = false;
            self.event_sender
                .send(ActivityEvent::ActivityResumed)
                .context("failed to send activity resumed through channel")?;
        }

        self.event_sender
            .send(ActivityEvent::MouseActivity { time_stamp: now })
            .context("failed to send mouse event")?;
        Ok(())
    }

    pub fn start_idle_monitoring(&self) {
        let state = self.state.clone();
        let event_sender = self.event_sender.clone();
        let idle_threshold = self.idle_threshold;

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(10));

                let mut state_guard = state.write().expect("failed to get a write lock on state");
                if let Some(activity) = state_guard.last_activity {
                    if let Ok(elapsed) = activity.elapsed() {
                        if elapsed >= idle_threshold && !state_guard.is_idle {
                            state_guard.is_idle = true;
                            event_sender
                                .send(ActivityEvent::IdleDetected { duration: elapsed })
                                .expect("failed to send idle detection through channel");
                        }
                    }
                }
            }
        });
    }
}
