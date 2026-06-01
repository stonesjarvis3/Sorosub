//! Command-line interface for the Glassbox incremental trace refresh tool.

use glassbox::{
    ContractState, IncrementalRefresher, InteractiveViewer, RefreshStrategy,
    TraceNode, TraceTree,
};

fn main() {
    println!("Glassbox - Incremental Trace Refresh Demo");
    println!("==========================================\n");

    // Create a sample trace tree
    let tree = create_sample_trace();
    println!("Initial trace tree created with {} nodes\n", tree.node_count());

    // Create an interactive viewer
    let mut viewer = InteractiveViewer::new(tree.clone());
    println!("Initial trace:");
    println!("{}", viewer.render());

    // Create initial contract state
    let mut state = ContractState::new(vec![1, 2, 3, 4], "initial_code_hash".to_string());
    state.set_ledger_entry(vec![10, 20], b"initial_value");
    state.set_ledger_entry(vec![30, 40], b"another_value");

    // Create incremental refresher
    let mut refresher = IncrementalRefresher::with_state(state.clone(), RefreshStrategy::Minimal);
    println!("Incremental refresher initialized with Minimal strategy\n");

    // Simulate a state change
    println!("Simulating ledger entry modification...");
    state.set_ledger_entry(vec![10, 20], b"modified_value");

    // Perform incremental refresh
    let result = refresher.refresh(viewer.tree_mut(), &state);
    println!("Refresh completed:");
    println!("  - Nodes refreshed: {}", result.nodes_refreshed);
    println!("  - Duration: {}ms", result.duration_ms);
    println!("  - Full replay: {}\n", result.full_replay);

    // Apply refresh to viewer
    viewer.apply_refresh(&result);
    
    // Show statistics
    let stats = viewer.statistics();
    println!("Viewer statistics:");
    println!("  - Total nodes: {}", stats.total_nodes);
    println!("  - Nodes needing refresh: {}", stats.nodes_needing_refresh);
    println!("  - Cached nodes: {}", stats.cached_nodes);
    println!("  - Update count: {}\n", stats.update_count);

    // Simulate code change (requires full replay)
    println!("Simulating contract code modification...");
    state.set_code_hash(b"new_contract_code");

    let result = refresher.refresh(viewer.tree_mut(), &state);
    println!("Refresh completed:");
    println!("  - Nodes refreshed: {}", result.nodes_refreshed);
    println!("  - Duration: {}ms", result.duration_ms);
    println!("  - Full replay: {}\n", result.full_replay);

    viewer.apply_refresh(&result);

    println!("Demo completed successfully!");
}

fn create_sample_trace() -> TraceTree {
    let mut tree = TraceTree::new();

    // Create root node
    let mut root = TraceNode::new(0, "contract_invoke".to_string());
    root.details
        .insert("contract_id".to_string(), "CDABCD...".to_string());
    root.details
        .insert("function".to_string(), "transfer".to_string());
    root.add_accessed_key(vec![10, 20]);
    root.depends_on_code = true;
    let root_id = tree.add_node(root);

    // Create child nodes
    let mut read_node = TraceNode::new(1, "storage_read".to_string());
    read_node.parent = Some(root_id);
    read_node.details.insert("key".to_string(), "balance".to_string());
    read_node.add_accessed_key(vec![10, 20]);
    let read_id = tree.add_node(read_node);

    let mut write_node = TraceNode::new(2, "storage_write".to_string());
    write_node.parent = Some(root_id);
    write_node.details
        .insert("key".to_string(), "balance".to_string());
    write_node.details
        .insert("value".to_string(), "1000".to_string());
    write_node.add_accessed_key(vec![10, 20]);
    let write_id = tree.add_node(write_node);

    let mut event_node = TraceNode::new(3, "emit_event".to_string());
    event_node.parent = Some(root_id);
    event_node.details
        .insert("event".to_string(), "Transfer".to_string());
    let event_id = tree.add_node(event_node);

    // Link children to root
    if let Some(root) = tree.get_node_mut(root_id) {
        root.add_child(read_id);
        root.add_child(write_id);
        root.add_child(event_id);
    }

    tree
}
