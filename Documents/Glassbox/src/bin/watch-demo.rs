//! Watch mode demonstration CLI.

use glassbox::{
    ContractState, DebugSession, IncrementalRefresher, InteractiveViewer, RefreshStrategy,
    TraceNode, TraceTree, WatchConfig, WatchMode,
};
use std::sync::{Arc, Mutex};

/// A simple debug session for demonstration
struct DemoDebugSession {
    name: String,
    state: Arc<Mutex<ContractState>>,
    refresher: Arc<Mutex<IncrementalRefresher>>,
    viewer: Arc<Mutex<InteractiveViewer>>,
    run_count: usize,
}

impl DemoDebugSession {
    fn new(name: &str) -> Self {
        let state = ContractState::new(vec![1, 2, 3, 4], "initial_code_hash".to_string());
        let tree = create_sample_trace();
        let refresher = IncrementalRefresher::with_state(state.clone(), RefreshStrategy::Minimal);
        let viewer = InteractiveViewer::new(tree);

        Self {
            name: name.to_string(),
            state: Arc::new(Mutex::new(state)),
            refresher: Arc::new(Mutex::new(refresher)),
            viewer: Arc::new(Mutex::new(viewer)),
            run_count: 0,
        }
    }
}

impl DebugSession for DemoDebugSession {
    fn run(&mut self) -> Result<(), String> {
        self.run_count += 1;

        println!("\n{}", "=".repeat(60));
        println!("Debug Session Run #{}", self.run_count);
        println!("{}\n", "=".repeat(60));

        // Simulate state change
        let mut state = self.state.lock().unwrap();
        state.set_ledger_entry(
            vec![10, 20],
            format!("value_{}", self.run_count).as_bytes(),
        );

        // Perform incremental refresh
        let mut refresher = self.refresher.lock().unwrap();
        let mut viewer = self.viewer.lock().unwrap();

        let result = refresher.refresh(viewer.tree_mut(), &state);

        println!("Refresh completed:");
        println!("  - Nodes refreshed: {}", result.nodes_refreshed);
        println!("  - Duration: {}ms", result.duration_ms);
        println!("  - Full replay: {}", result.full_replay);

        viewer.apply_refresh(&result);

        let stats = viewer.statistics();
        println!("\nViewer statistics:");
        println!("  - Total nodes: {}", stats.total_nodes);
        println!("  - Nodes needing refresh: {}", stats.nodes_needing_refresh);
        println!("  - Cached nodes: {}", stats.cached_nodes);

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

fn create_sample_trace() -> TraceTree {
    let mut tree = TraceTree::new();

    let mut root = TraceNode::new(0, "contract_invoke".to_string());
    root.details
        .insert("contract_id".to_string(), "CDABCD...".to_string());
    root.add_accessed_key(vec![10, 20]);
    root.depends_on_code = true;
    let root_id = tree.add_node(root);

    let mut read_node = TraceNode::new(1, "storage_read".to_string());
    read_node.parent = Some(root_id);
    read_node.add_accessed_key(vec![10, 20]);
    let read_id = tree.add_node(read_node);

    let mut write_node = TraceNode::new(2, "storage_write".to_string());
    write_node.parent = Some(root_id);
    write_node.add_accessed_key(vec![10, 20]);
    let write_id = tree.add_node(write_node);

    if let Some(root) = tree.get_node_mut(root_id) {
        root.add_child(read_id);
        root.add_child(write_id);
    }

    tree
}

fn main() {
    println!("Glassbox - Watch Mode Demo");
    println!("==========================\n");

    // Create watch configuration
    let mut config = WatchConfig::new();
    config
        .add_watch_path("src")
        .add_include_pattern("*.rs".to_string())
        .add_include_pattern("*.toml".to_string())
        .set_debounce_ms(1000)
        .set_run_on_startup(true);

    println!("Watch configuration:");
    println!("  - Watch paths: {:?}", config.watch_paths);
    println!("  - Include patterns: {:?}", config.include_patterns);
    println!("  - Exclude patterns: {:?}", config.exclude_patterns);
    println!("  - Debounce delay: {}ms", config.debounce_ms);
    println!("  - Run on startup: {}\n", config.run_on_startup);

    // Create debug session
    let session = DemoDebugSession::new("demo-session");

    // Create and start watch mode
    let mut watch = WatchMode::new(config);

    println!("Starting watch mode...");
    println!("Watching for file changes. Press Ctrl+C to stop.\n");

    // In a real application, this would run indefinitely
    // For demo purposes, we'll just show the setup
    match watch.start(session) {
        Ok(_) => {
            println!("\nWatch mode stopped.");

            let stats = watch.statistics();
            println!("\nWatch mode statistics:");
            println!("  - Total events: {}", stats.total_events);
            println!("  - Sessions triggered: {}", stats.sessions_triggered);
            println!("  - Sessions succeeded: {}", stats.sessions_succeeded);
            println!("  - Sessions failed: {}", stats.sessions_failed);
            println!(
                "  - Average session duration: {}ms",
                stats.avg_session_duration_ms
            );
        }
        Err(e) => {
            eprintln!("Error starting watch mode: {}", e);
        }
    }
}
