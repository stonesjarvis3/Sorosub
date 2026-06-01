# Glassbox - Incremental Trace Refresh

A Rust library for incrementally refreshing contract execution traces when state changes occur during debugging, avoiding the need for full replay.

## Features

- **State Change Detection**: Automatically detects changes in contract ledger entries and code artifacts
- **Incremental Refresh**: Updates only affected portions of the trace tree
- **Multiple Refresh Strategies**: Choose between minimal, with-children, with-descendants, or full refresh
- **Interactive Viewer**: Visualize traces with support for incremental updates
- **Performance Optimized**: Avoids unnecessary recomputation by tracking dependencies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
glassbox = "0.1.0"
```

## Quick Start

```rust
use glassbox::{
    ContractState, IncrementalRefresher, InteractiveViewer,
    RefreshStrategy, TraceTree, TraceNode
};

// Create a trace tree
let mut tree = TraceTree::new();
let mut root = TraceNode::new(0, "invoke".to_string());
root.add_accessed_key(vec![1, 2, 3]);
tree.add_node(root);

// Initialize contract state
let mut state = ContractState::new(vec![1, 2, 3, 4], "code_hash".to_string());
state.set_ledger_entry(vec![1, 2, 3], b"initial_value");

// Create refresher and viewer
let mut refresher = IncrementalRefresher::with_state(
    state.clone(),
    RefreshStrategy::Minimal
);
let mut viewer = InteractiveViewer::new(tree);

// Modify state
state.set_ledger_entry(vec![1, 2, 3], b"new_value");

// Perform incremental refresh
let result = refresher.refresh(viewer.tree_mut(), &state);
viewer.apply_refresh(&result);

// Render updated trace
println!("{}", viewer.render());
```

## Architecture

### State Change Detection

The `StateChangeDetector` monitors contract state and identifies:
- **Ledger Entry Changes**: Additions, modifications, and removals
- **Code Artifact Changes**: Updates to contract code

Changes are detected by computing SHA-256 hashes of state data and comparing them between snapshots.

### Trace Tree Structure

The `TraceTree` represents execution traces as a tree of `TraceNode` objects. Each node:
- Tracks which state keys it accesses
- Records whether it depends on contract code
- Maintains parent-child relationships
- Can be marked for refresh when dependencies change

### Incremental Refresh

The `IncrementalRefresher` implements four refresh strategies:

1. **Minimal**: Refresh only nodes that directly access changed state
2. **WithChildren**: Refresh affected nodes and their immediate children
3. **WithDescendants**: Refresh affected nodes and all descendants
4. **Full**: Replay the entire trace (used when contract code changes)

### Interactive Viewer

The `InteractiveViewer` provides:
- Multiple display formats (Compact, Detailed, JSON)
- Display caching for performance
- Update history tracking
- Statistics about the current view

## Refresh Workflow

1. **Initialize**: Create a `StateChangeDetector` with the initial contract state
2. **Detect Changes**: When state changes, call `detect_changes()` to identify modifications
3. **Mark Affected Nodes**: The trace tree marks nodes that access changed state
4. **Apply Strategy**: The refresh strategy determines which nodes to update
5. **Update Viewer**: The viewer refreshes only the affected portions of the display

## Examples

### Basic Incremental Refresh

```rust
use glassbox::*;

let mut state = ContractState::new(vec![1], "hash".to_string());
let mut refresher = IncrementalRefresher::with_state(
    state.clone(),
    RefreshStrategy::Minimal
);

// ... create and populate trace tree ...

// Modify state
state.set_ledger_entry(vec![10, 20], b"new_value");

// Refresh only affected nodes
let result = refresher.refresh(&mut tree, &state);
println!("Refreshed {} nodes in {}ms", 
    result.nodes_refreshed, 
    result.duration_ms
);
```

### Handling Code Changes

```rust
// Code changes require full replay
state.set_code_hash(b"new_contract_code");

let result = refresher.refresh(&mut tree, &state);
assert!(result.full_replay);
```

### Custom Display Format

```rust
let mut viewer = InteractiveViewer::new(tree);

// Use compact format
viewer.set_format(DisplayFormat::Compact);
println!("{}", viewer.render());

// Switch to JSON format
viewer.set_format(DisplayFormat::Json);
println!("{}", viewer.render());
```

### Show Only Changed Nodes

```rust
viewer.set_show_only_changed(true);
let output = viewer.render();
// Only nodes marked for refresh are displayed
```

## Testing

Run the test suite:

```bash
cargo test
```

Run the CLI demo:

```bash
cargo run --bin glassbox-cli
```

## Performance Considerations

- **State Hashing**: Uses SHA-256 for reliable change detection
- **Display Caching**: Viewer caches formatted output to avoid redundant rendering
- **Minimal Recomputation**: Only affected trace segments are reprocessed
- **Strategy Selection**: Choose the appropriate refresh strategy for your use case

## API Documentation

Generate and view the full API documentation:

```bash
cargo doc --open
```

## Implementation Details

### State Change Detection

The detector maintains a snapshot of the previous state and compares it with the new state:

```rust
pub fn detect_changes(&mut self, new_state: &ContractState) -> Vec<StateChange>
```

Returns a vector of `StateChange` enums describing what changed.

### Trace Node Dependencies

Each `TraceNode` tracks:
- `accessed_keys`: State keys read or written by this operation
- `depends_on_code`: Whether the node's behavior depends on contract code

When state changes, nodes are marked for refresh based on these dependencies.

### Refresh Strategies

The strategy determines the scope of refresh:

- **Minimal**: Best for isolated state changes
- **WithChildren**: Use when child operations may be affected
- **WithDescendants**: Use when changes cascade through the tree
- **Full**: Required for code changes or when dependencies are unclear

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)

## License

MIT License - see LICENSE file for details

## Related Issues

- Closes #130: Implement incremental trace refresh support
