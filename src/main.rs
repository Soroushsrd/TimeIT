mod file_session;
mod file_watcher;
mod input_watcher;
mod manager;
mod stats;
mod tracking_event;

use crate::input_watcher::InputMonitor;
use file_watcher::FileWatcher;
use notify::{Event, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::sync::Arc;

//WARNING:use a macro for logging and a thread local buffer
//################################################################

#[tokio::main]
async fn main() {
    println!("Starting filtered file watcher...");
    println!("Watching current directory for source code changes");

    // we create an input monitor and its receiver channel
    let (input_monitor, receiver) = InputMonitor::new();
    // wrap an arc around it so that we could pass it around in threads
    let input_monitor = Arc::new(input_monitor);

    // one taks to monitor idle activity
    // one task to receive events
    tokio::spawn(input_monitor.clone().start_idle_monitoring(20));
    tokio::spawn(input_monitor.clone().receive_events(receiver));

    let (tx, rx) = crossbeam::channel::bounded(10);
    let rx = Arc::new(rx);
    let tx_clone = tx.clone();
    let handler = move |event: Result<Event>| {
        let _ = tx_clone.send(event);
    };
    let mut watcher = notify::recommended_watcher(handler).unwrap();

    watcher
        .watch(Path::new("."), RecursiveMode::Recursive)
        .expect("failed to watch using watcher!");

    tokio::spawn(async move {
        let mut file_watcher = FileWatcher::new();
        file_watcher.handle_file_watcher(rx.clone()).await;
    });

    tokio::spawn(input_monitor.start_activity_monitoring());

    println!("Press Ctrl+C to stop\n");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
