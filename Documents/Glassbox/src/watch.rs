//! Watch mode for automatic debug session reruns.
//!
//! This module provides file watching capabilities that automatically
//! rerun debug sessions when relevant source or config files are modified.

use crossbeam_channel::{bounded, select, Receiver, Sender};
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Represents a file change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchEvent {
    /// A file was modified
    FileModified { path: PathBuf },
    /// A file was created
    FileCreated { path: PathBuf },
    /// A file was deleted
    FileDeleted { path: PathBuf },
    /// Debug session started
    SessionStarted,
    /// Debug session completed
    SessionCompleted { duration_ms: u64 },
    /// Debug session failed
    SessionFailed { error: String },
}

/// Configuration for watch mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Paths to watch for changes
    pub watch_paths: Vec<PathBuf>,
    /// File patterns to include (e.g., "*.rs", "*.toml")
    pub include_patterns: Vec<String>,
    /// File patterns to exclude (e.g., "target/*", "*.tmp")
    pub exclude_patterns: Vec<String>,
    /// Debounce delay in milliseconds
    pub debounce_ms: u64,
    /// Whether to run on startup
    pub run_on_startup: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![PathBuf::from(".")],
            include_patterns: vec!["*.rs".to_string(), "*.toml".to_string()],
            exclude_patterns: vec!["target/*".to_string(), ".*".to_string()],
            debounce_ms: 500,
            run_on_startup: true,
        }
    }
}

impl WatchConfig {
    /// Creates a new watch configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a path to watch
    pub fn add_watch_path<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.watch_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Adds an include pattern
    pub fn add_include_pattern(&mut self, pattern: String) -> &mut Self {
        self.include_patterns.push(pattern);
        self
    }

    /// Adds an exclude pattern
    pub fn add_exclude_pattern(&mut self, pattern: String) -> &mut Self {
        self.exclude_patterns.push(pattern);
        self
    }

    /// Sets the debounce delay
    pub fn set_debounce_ms(&mut self, ms: u64) -> &mut Self {
        self.debounce_ms = ms;
        self
    }

    /// Sets whether to run on startup
    pub fn set_run_on_startup(&mut self, run: bool) -> &mut Self {
        self.run_on_startup = run;
        self
    }

    /// Checks if a path should be watched based on patterns
    pub fn should_watch(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check exclude patterns first
        for pattern in &self.exclude_patterns {
            if Self::matches_pattern(&path_str, pattern) {
                return false;
            }
        }

        // Check include patterns
        if self.include_patterns.is_empty() {
            return true;
        }

        for pattern in &self.include_patterns {
            if Self::matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let (prefix, suffix) = (parts[0], parts[1]);
                return path.starts_with(prefix) && path.ends_with(suffix);
            }
        }
        path == pattern
    }
}

/// Represents a debug session that can be rerun
pub trait DebugSession: Send {
    /// Runs the debug session
    fn run(&mut self) -> Result<(), String>;

    /// Gets the session name
    fn name(&self) -> &str;
}

/// Statistics about watch mode execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchStatistics {
    /// Total number of file events received
    pub total_events: usize,
    /// Number of debug sessions triggered
    pub sessions_triggered: usize,
    /// Number of successful sessions
    pub sessions_succeeded: usize,
    /// Number of failed sessions
    pub sessions_failed: usize,
    /// Number of debounced events
    pub events_debounced: usize,
    /// Average session duration in milliseconds
    pub avg_session_duration_ms: u64,
}

impl WatchStatistics {
    fn new() -> Self {
        Self {
            total_events: 0,
            sessions_triggered: 0,
            sessions_succeeded: 0,
            sessions_failed: 0,
            events_debounced: 0,
            avg_session_duration_ms: 0,
        }
    }
}

/// Watch mode manager
pub struct WatchMode {
    config: WatchConfig,
    event_history: Arc<Mutex<Vec<WatchEvent>>>,
    statistics: Arc<Mutex<WatchStatistics>>,
    stop_signal: Option<Sender<()>>,
}

impl WatchMode {
    /// Creates a new watch mode instance
    pub fn new(config: WatchConfig) -> Self {
        Self {
            config,
            event_history: Arc::new(Mutex::new(Vec::new())),
            statistics: Arc::new(Mutex::new(WatchStatistics::new())),
            stop_signal: None,
        }
    }

    /// Starts watching files and running debug sessions
    pub fn start<S: DebugSession + 'static>(
        &mut self,
        mut session: S,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = bounded(100);
        let (stop_tx, stop_rx) = bounded(1);
        self.stop_signal = Some(stop_tx);

        // Set up file watcher
        let mut watcher = RecommendedWatcher::new(
            {
                let tx = tx.clone();
                move |res: NotifyResult<Event>| {
                    if let Ok(event) = res {
                        let _ = tx.send(event);
                    }
                }
            },
            Config::default(),
        )?;

        // Watch all configured paths
        for path in &self.config.watch_paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }

        // Run on startup if configured
        if self.config.run_on_startup {
            self.run_session(&mut session);
        }

        // Start event processing loop
        self.process_events(rx, stop_rx, session)?;

        Ok(())
    }

    /// Stops the watch mode
    pub fn stop(&mut self) {
        if let Some(stop_tx) = self.stop_signal.take() {
            let _ = stop_tx.send(());
        }
    }

    /// Gets the event history
    pub fn event_history(&self) -> Vec<WatchEvent> {
        self.event_history.lock().unwrap().clone()
    }

    /// Gets the statistics
    pub fn statistics(&self) -> WatchStatistics {
        self.statistics.lock().unwrap().clone()
    }

    /// Clears the event history
    pub fn clear_history(&mut self) {
        self.event_history.lock().unwrap().clear();
    }

    /// Processes file system events with debouncing
    fn process_events<S: DebugSession>(
        &self,
        rx: Receiver<Event>,
        stop_rx: Receiver<()>,
        mut session: S,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_run = Instant::now();
        let mut pending_changes: HashSet<PathBuf> = HashSet::new();
        let debounce_duration = Duration::from_millis(self.config.debounce_ms);

        loop {
            select! {
                recv(rx) -> event => {
                    if let Ok(event) = event {
                        self.handle_file_event(event, &mut pending_changes);
                    }
                }
                recv(stop_rx) -> _ => {
                    break;
                }
                default(Duration::from_millis(100)) => {
                    // Check if we should trigger a rerun
                    if !pending_changes.is_empty() && last_run.elapsed() >= debounce_duration {
                        self.run_session(&mut session);
                        pending_changes.clear();
                        last_run = Instant::now();
                    }
                }
            }
        }

        Ok(())
    }

    /// Handles a file system event
    fn handle_file_event(&self, event: Event, pending_changes: &mut HashSet<PathBuf>) {
        let mut stats = self.statistics.lock().unwrap();
        stats.total_events += 1;
        drop(stats);

        for path in &event.paths {
            if !self.config.should_watch(path) {
                continue;
            }

            let watch_event = match event.kind {
                EventKind::Modify(_) => WatchEvent::FileModified {
                    path: path.clone(),
                },
                EventKind::Create(_) => WatchEvent::FileCreated {
                    path: path.clone(),
                },
                EventKind::Remove(_) => WatchEvent::FileDeleted {
                    path: path.clone(),
                },
                _ => continue,
            };

            self.event_history.lock().unwrap().push(watch_event);
            pending_changes.insert(path.clone());
        }
    }

    /// Runs a debug session
    fn run_session<S: DebugSession>(&self, session: &mut S) {
        let start_time = Instant::now();

        // Record session start
        self.event_history
            .lock()
            .unwrap()
            .push(WatchEvent::SessionStarted);

        let mut stats = self.statistics.lock().unwrap();
        stats.sessions_triggered += 1;
        drop(stats);

        // Run the session
        match session.run() {
            Ok(_) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;

                self.event_history
                    .lock()
                    .unwrap()
                    .push(WatchEvent::SessionCompleted { duration_ms });

                let mut stats = self.statistics.lock().unwrap();
                stats.sessions_succeeded += 1;

                // Update average duration
                let total_duration = stats.avg_session_duration_ms * (stats.sessions_succeeded - 1) as u64
                    + duration_ms;
                stats.avg_session_duration_ms = total_duration / stats.sessions_succeeded as u64;
            }
            Err(error) => {
                self.event_history
                    .lock()
                    .unwrap()
                    .push(WatchEvent::SessionFailed {
                        error: error.clone(),
                    });

                let mut stats = self.statistics.lock().unwrap();
                stats.sessions_failed += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestSession {
        name: String,
        run_count: Arc<AtomicUsize>,
        should_fail: bool,
    }

    impl TestSession {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                run_count: Arc::new(AtomicUsize::new(0)),
                should_fail: false,
            }
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        fn run_count(&self) -> usize {
            self.run_count.load(Ordering::SeqCst)
        }
    }

    impl DebugSession for TestSession {
        fn run(&mut self) -> Result<(), String> {
            self.run_count.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                Err("Test failure".to_string())
            } else {
                Ok(())
            }
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert_eq!(config.debounce_ms, 500);
        assert!(config.run_on_startup);
        assert!(!config.watch_paths.is_empty());
    }

    #[test]
    fn test_watch_config_builder() {
        let mut config = WatchConfig::new();
        config
            .add_watch_path("src")
            .add_include_pattern("*.rs".to_string())
            .add_exclude_pattern("target/*".to_string())
            .set_debounce_ms(1000)
            .set_run_on_startup(false);

        assert_eq!(config.debounce_ms, 1000);
        assert!(!config.run_on_startup);
        assert!(config.watch_paths.iter().any(|p| p.to_str() == Some("src")));
    }

    #[test]
    fn test_should_watch_include_patterns() {
        let mut config = WatchConfig::new();
        config.include_patterns = vec!["*.rs".to_string()];
        config.exclude_patterns = vec![];

        assert!(config.should_watch(Path::new("src/main.rs")));
        assert!(!config.should_watch(Path::new("Cargo.toml")));
    }

    #[test]
    fn test_should_watch_exclude_patterns() {
        let mut config = WatchConfig::new();
        config.include_patterns = vec![];
        config.exclude_patterns = vec!["target/*".to_string()];

        assert!(!config.should_watch(Path::new("target/debug/main")));
        assert!(config.should_watch(Path::new("src/main.rs")));
    }

    #[test]
    fn test_pattern_matching() {
        assert!(WatchConfig::matches_pattern("test.rs", "*.rs"));
        assert!(WatchConfig::matches_pattern("src/main.rs", "*.rs"));
        assert!(!WatchConfig::matches_pattern("test.toml", "*.rs"));
        assert!(WatchConfig::matches_pattern("exact.txt", "exact.txt"));
    }

    #[test]
    fn test_watch_mode_creation() {
        let config = WatchConfig::default();
        let watch = WatchMode::new(config);

        let stats = watch.statistics();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.sessions_triggered, 0);
    }

    #[test]
    fn test_event_history() {
        let config = WatchConfig::default();
        let mut watch = WatchMode::new(config);

        watch
            .event_history
            .lock()
            .unwrap()
            .push(WatchEvent::SessionStarted);

        let history = watch.event_history();
        assert_eq!(history.len(), 1);
        assert!(matches!(history[0], WatchEvent::SessionStarted));

        watch.clear_history();
        assert_eq!(watch.event_history().len(), 0);
    }

    #[test]
    fn test_session_run_success() {
        let config = WatchConfig::new();
        let watch = WatchMode::new(config);
        let mut session = TestSession::new("test");

        watch.run_session(&mut session);

        assert_eq!(session.run_count(), 1);
        let stats = watch.statistics();
        assert_eq!(stats.sessions_triggered, 1);
        assert_eq!(stats.sessions_succeeded, 1);
        assert_eq!(stats.sessions_failed, 0);
    }

    #[test]
    fn test_session_run_failure() {
        let config = WatchConfig::new();
        let watch = WatchMode::new(config);
        let mut session = TestSession::new("test").with_failure();

        watch.run_session(&mut session);

        assert_eq!(session.run_count(), 1);
        let stats = watch.statistics();
        assert_eq!(stats.sessions_triggered, 1);
        assert_eq!(stats.sessions_succeeded, 0);
        assert_eq!(stats.sessions_failed, 1);
    }

    #[test]
    fn test_multiple_sessions() {
        let config = WatchConfig::new();
        let watch = WatchMode::new(config);
        let mut session = TestSession::new("test");

        watch.run_session(&mut session);
        watch.run_session(&mut session);
        watch.run_session(&mut session);

        assert_eq!(session.run_count(), 3);
        let stats = watch.statistics();
        assert_eq!(stats.sessions_triggered, 3);
        assert_eq!(stats.sessions_succeeded, 3);
    }

    #[test]
    fn test_watch_event_serialization() {
        let event = WatchEvent::FileModified {
            path: PathBuf::from("test.rs"),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WatchEvent = serde_json::from_str(&json).unwrap();

        match deserialized {
            WatchEvent::FileModified { path } => {
                assert_eq!(path, PathBuf::from("test.rs"));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_statistics_average_duration() {
        let config = WatchConfig::new();
        let watch = WatchMode::new(config);

        // Simulate sessions with known durations
        {
            let mut stats = watch.statistics.lock().unwrap();
            stats.sessions_triggered = 3;
            stats.sessions_succeeded = 3;
            stats.avg_session_duration_ms = (100 + 200 + 300) / 3; // 200ms average
        }

        let stats = watch.statistics();
        assert_eq!(stats.avg_session_duration_ms, 200);
    }
}
