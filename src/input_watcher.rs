use anyhow::{Context, Result};
use rdev::{Event, listen};
use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};
use tokio::sync::broadcast::{self, Receiver, Sender};
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

#[allow(dead_code)]
impl ActivityState {
    pub fn new() -> Self {
        Self {
            last_activity: None,
            is_idle: false,
        }
    }

    /// checks to see if there has been some sort of keyboard or mouse
    /// activity within the specified duration
    pub fn is_recently_active(&self, within: Duration) -> bool {
        if let Some(activity) = self.last_activity {
            if let Ok(elapsed) = activity.elapsed() {
                return elapsed <= within;
            }
        }
        false
    }

    /// returns the time dif between now and last activity
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
    pub event_sender: Sender<ActivityEvent>,
    idle_threshold: Duration,
}

#[allow(dead_code)]
impl InputMonitor {
    /// takes in the duration for idle threshold and a sender channel for events
    pub fn new() -> (Self, Receiver<ActivityEvent>) {
        let (tx, rx) = broadcast::channel::<ActivityEvent>(100);
        (
            Self {
                state: Arc::new(RwLock::new(ActivityState::new())),
                event_sender: tx,
                idle_threshold: Duration::from_secs(20),
            },
            rx,
        )
    }

    /// returns ActivityState of the object
    pub fn get_state(&self) -> Option<ActivityState> {
        match self.state.read() {
            Ok(event) => Some(event.clone()),
            Err(e) => {
                error!("Failed to get activity state: {e}");
                None
            }
        }
    }

    /// if there has been some keyboard activity, this method must be called
    /// it will set the last activity to now, is_idle to false and send a
    /// ActivityEvent::ActivityResumed through the channel
    /// if is_idle is already false, a KeyboardActivity will be sent back
    pub fn handle_keyboard_event(&self) -> Result<()> {
        println!("inside keyboard handler method");

        let now = SystemTime::now();
        let mut state = self
            .state
            .write()
            .expect("failed to get a read lock on state");
        state.last_activity = Some(now);

        if state.is_idle {
            state.is_idle = false;
            println!("activity resumed");
            self.event_sender
                .send(ActivityEvent::ActivityResumed)
                .context("failed to send activity resumed through channel")?;
        }

        println!("keyboard activity detected");
        self.event_sender
            .send(ActivityEvent::KeyboardActivity { time_stamp: now })
            .context("failed to send keyboard event")?;
        Ok(())
    }

    /// if a mouse activity is detected, last activity will be set to now()
    /// then if is_idle is true, we'll set it to false and send back an
    /// ActivityResumed event, otherwise a MouseActivity event will be returned
    pub fn handle_mouse_event(&self) -> Result<()> {
        println!("inside keyboard handler method");
        let now = SystemTime::now();
        let mut state = self
            .state
            .write()
            .expect("failed to get a read lock on state");
        state.last_activity = Some(now);

        if state.is_idle {
            state.is_idle = false;

            println!("activity resumed");
            self.event_sender
                .send(ActivityEvent::ActivityResumed)
                .context("failed to send activity resumed through channel")?;
        }

        println!("mouse activity detected");
        self.event_sender
            .send(ActivityEvent::MouseActivity { time_stamp: now })
            .context("failed to send mouse event")?;
        Ok(())
    }

    pub async fn start_activity_monitoring(self: Arc<Self>) {
        let input_monitor = Arc::new(RwLock::new(self));

        let callback = move |event: Event| match event.event_type {
            rdev::EventType::KeyPress(_key) | rdev::EventType::KeyRelease(_key) => {
                let lock = input_monitor
                    .read()
                    .expect("failed to get a read lock on input monitor");
                lock.handle_keyboard_event()
                    .expect("failed to handle keyboard event");
            }
            rdev::EventType::MouseMove { x: _, y: _ } => {
                let lock = input_monitor
                    .read()
                    .expect("failed to get a read lock on input monitor");
                lock.handle_mouse_event().unwrap()
            }
            _ => {}
        };

        if let Err(e) = listen(callback) {
            println!("error: {:?}", e);
        }
    }

    pub async fn receive_events(self: Arc<Self>, mut receiver: Receiver<ActivityEvent>) {
        while let Ok(event) = receiver.recv().await {
            match event {
                ActivityEvent::KeyboardActivity { time_stamp } => {
                    println!("Keyboard activity at {:?}", time_stamp);
                }
                ActivityEvent::MouseActivity { time_stamp } => {
                    println!("Mouse activity at {:?}", time_stamp);
                }
                ActivityEvent::IdleDetected { duration } => {
                    println!("User idle for {:?}", duration);
                }
                ActivityEvent::ActivityResumed => {
                    println!("User activity resumed");
                }
            }
        }
    }
    /// spawns a thread inside which idle activity is detected.
    /// if a file is not modified or no keyboard/mouse activity
    /// is detected within idle_threshold, activity state will be
    /// set to idle and an IdleDetected event will be sent back
    /// monitor_tick parameter will dictate the frequency of this check
    pub async fn start_idle_monitoring(self: Arc<Self>, monitor_tick: u64) {
        let state = self.state.clone();
        let event_sender = self.event_sender.clone();
        let idle_threshold = self.idle_threshold;

        loop {
            tokio::time::sleep(Duration::from_secs(monitor_tick)).await;

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
    }
}
