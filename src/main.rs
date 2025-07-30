mod file_watcher;
mod input_watcher;
use crate::input_watcher::InputMonitor;
use crossbeam::channel::unbounded;
use file_watcher::FileWatcher;
use notify::{Event, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::sync::{Arc, mpsc};
use tokio;

//WARNING: what needs to be done:
//      idle monitoring in one thread
//      event receiving in another thread
//      listening and file watcher in the main thread

//WARNING: it seems that crossbeam channels can be used for file watcher. use that
//      instead of tokio. should be able to send them accross threads
//
//WARNING:use a macro for logging and a thread local buffer

//################################################################

#[tokio::main]
async fn main() {
    println!("Starting filtered file watcher...");
    println!("Watching current directory for source code changes");

    let (input_monitor, receiver) = InputMonitor::new();
    let input_monitor = Arc::new(input_monitor);

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
