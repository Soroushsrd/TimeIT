use crossbeam::channel::Receiver;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use notify::Event;

/// file watcher is used to keep track of file events
/// it will keep a map of last events and the time they happened
/// it will also keep track of the file extension in which the
/// modifications occur and will ignore the files for which ignore
/// patterns apply
pub struct FileWatcher {
    //TODO: use atomics? with rwlock?
    last_events: HashMap<PathBuf, Instant>,
    debouncing_duration: Duration,
    source_extensions: HashSet<&'static str>,
    ignore_patterns: Vec<&'static str>,
}

impl FileWatcher {
    /// creates a new file watcher. doesnt take anything as input for now
    pub fn new() -> Self {
        let source_extensions = HashSet::from([
            ".rs", ".py", ".js", ".jsx", ".ts", ".tsx", ".go", ".cpp", ".h", ".c", ".lua",
        ]);

        let ignore_patterns = vec![
            // Build artifacts
            "/target/",
            "/build/",
            "/dist/",
            "/out/",
            "/.next/",
            // Dependencies
            "/node_modules/",
            "/vendor/",
            "/.venv/",
            "/venv/",
            "/env/",
            // Version control
            "/.git/",
            "/.svn/",
            "/.hg/",
            // IDE/Editor files
            "/.vscode/",
            "/.idea/",
            "/.vs/",
            // Temporary files
            "/tmp/",
            "/temp/",
            "/.tmp/",
            // Cache directories
            "/.cache/",
            "/cache/",
            "/__pycache__/",
            "/.pytest_cache/",
            // Log files
            "/logs/",
            "/.log/",
        ];

        Self {
            last_events: HashMap::new(),
            debouncing_duration: Duration::from_millis(100),
            source_extensions,
            ignore_patterns,
        }
    }

    /// checks to see if the event received is actually a modification
    /// type of event. if yes, then returns the path of the modified file
    pub fn process_event(&mut self, event: &Event) -> Option<PathBuf> {
        match event.kind {
            notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )) => {
                // will check for keyboard movement later
                for path in &event.paths {
                    if self.should_ignore(path) {
                        continue;
                    } else if self.should_debounce(path) {
                        continue;
                    } else {
                        return Some(path.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// checks and sees if the diff in time between the new event
    /// and last event is less than debouncing duration.
    /// also saves the new event instant if time diff is
    /// more than debouncing duration
    fn should_debounce(&mut self, path: &Path) -> bool {
        let now = Instant::now();
        let path = path.to_path_buf();
        if let Some(last_event) = self.last_events.get(&path) {
            if now.duration_since(*last_event) < self.debouncing_duration {
                return true;
            }
        }
        self.last_events.insert(path, now);
        false
    }

    // checks and sees if the path contains modifications
    // in files that was mentioned in ignored paths.
    // this should later be used to check .ignore files(?)
    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.ignore_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        // checking file extensions
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_with_dot = format!(".{}", ext_str);
                if self.source_extensions.contains(ext_with_dot.as_str()) {
                    return false;
                }
            }
        }
        true
    }
    pub async fn handle_file_watcher(&mut self, rx: Arc<Receiver<crate::Result<Event>>>) {
        for res in rx.as_ref() {
            match res {
                Ok(event) => {
                    if let Some(path) = self.process_event(&event) {
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
    }
}

// TODO: use an enum
pub fn detect_language(path: &Path) -> Option<&'static str> {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            match ext_str {
                "rs" => Some("Rust"),
                "py" => Some("Python"),
                "js" => Some("JavaScript"),
                "ts" => Some("TypeScript"),
                "jsx" => Some("JavaScript React"),
                "tsx" => Some("TypeScript React"),
                "go" => Some("Go"),
                "cpp" | "cc" | "cxx" => Some("C++"),
                "c" => Some("C"),
                "h" | "hpp" => Some("C/C++ Header"),
                "java" => Some("Java"),
                "kt" => Some("Kotlin"),
                "php" => Some("PHP"),
                "rb" => Some("Ruby"),
                "swift" => Some("Swift"),
                "scala" => Some("Scala"),
                "clj" => Some("Clojure"),
                "hs" => Some("Haskell"),
                "elm" => Some("Elm"),
                "dart" => Some("Dart"),
                "lua" => Some("Lua"),
                "vim" => Some("Vim Script"),
                "sh" | "bash" => Some("Bash"),
                "zsh" => Some("Zsh"),
                "fish" => Some("Fish"),
                "ps1" => Some("PowerShell"),
                "sql" => Some("SQL"),
                "html" => Some("HTML"),
                "css" => Some("CSS"),
                "scss" => Some("SCSS"),
                "sass" => Some("Sass"),
                "less" => Some("Less"),
                "md" => Some("Markdown"),
                "yml" | "yaml" => Some("YAML"),
                "toml" => Some("TOML"),
                "json" => Some("JSON"),
                "xml" => Some("XML"),
                _ => Some("Unknown"),
            }
        } else {
            None
        }
    } else {
        // handling files without extensions
        if let Some(filename) = path.file_name() {
            if let Some(name_str) = filename.to_str() {
                match name_str {
                    "Dockerfile" => Some("Docker"),
                    "Makefile" => Some("Makefile"),
                    "CMakeLists.txt" => Some("CMake"),
                    _ => Some("Text"),
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
