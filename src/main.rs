mod file_watcher;
mod input_watcher;
use crate::input_watcher::InputMonitor;
use file_watcher::FileWatcher;
use notify::{Event, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::sync::{Arc, mpsc};
use tokio;

#[tokio::main]
async fn main() {
    //TODO: use a macro for logging and a thread local buffer

    println!("Starting filtered file watcher...");
    println!("Watching current directory for source code changes");
    println!("Press Ctrl+C to stop\n");

    let (input_monitor, receiver) = InputMonitor::new();
    let input_monitor = Arc::new(input_monitor);
    // tokio::spawn(input_monitor.clone().start_activity_monitoring());
    tokio::spawn(input_monitor.clone().start_idle_monitoring(20));
    tokio::spawn(input_monitor.receive_events(receiver));

    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let rx = Arc::new(rx);
    let mut watcher = notify::recommended_watcher(tx.clone()).unwrap();
    let mut file_watcher = FileWatcher::new();
    // //TODO: for now it just listens to the local directory
    // //we should add global monitoring

    watcher
        .watch(Path::new("."), RecursiveMode::Recursive)
        .expect("failed to watch using watcher!");

    file_watcher.handle_file_watcher(&rx).await;
    //
    // loop {
    //     tokio::select! {
    //        _ = file_watcher.handle_file_watcher(&rx)=>{}
    //        _ = input_monitor.clone().start_idle_monitoring(20)=>{}
    //        _ = input_monitor.clone().receive_events(input_monitor.event_sender.subscribe())=>{}
    //        _ = input_monitor.clone().start_activity_monitoring()=>{}
    //     }
    // }
}
