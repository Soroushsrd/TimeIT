mod file_watcher;
mod input_watcher;
use file_watcher::{FileWatcher, detect_language};
use notify::{Event, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::sync::mpsc;

fn main() -> Result<()> {
    println!("Starting filtered file watcher...");
    println!("Watching current directory for source code changes");
    println!("Press Ctrl+C to stop\n");
    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    let mut file_watcher = FileWatcher::new();
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if let Some(path) = file_watcher.process_event(&event) {
                    let language = detect_language(&path).unwrap_or("unknown");
                    let relative_path = path.strip_prefix("./").unwrap_or(&path);

                    println!("file modified: {} ({language})", relative_path.display());
                }
            }
            Err(e) => {
                println!("file watcher error: {e}");
            }
        }
    }
    Ok(())
}
