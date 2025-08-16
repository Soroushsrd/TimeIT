use tokio::time::Duration;

#[derive(Debug, Clone)]
pub enum TrackingEvents {
    FileOpened { path: String, language: String },
    FileClosed { path: String },
    FileModified { path: String },
    FileFocused { path: String },

    UserActive,
    UserIdle { duration: Duration },

    SystemAwake,
    SystemSleep,
}
