//! Incremental trace refresh implementation.
//!
//! This module provides the core logic for incrementally refreshing
//! trace output when contract state changes, avoiding full replay.

use crate::state::{ContractState, StateChange, StateChangeDetector};
use crate::trace::{NodeId, TraceTree};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Strategy for refreshing the trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefreshStrategy {
    /// Refresh only the minimal set of affected nodes
    Minimal,
    /// Refresh affected nodes and their immediate children
    WithChildren,
    /// Refresh affected nodes and all descendants
    WithDescendants,
    /// Full replay of the entire trace
    Full,
}

/// Result of a refresh operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResult {
    /// Number of nodes that were refreshed
    pub nodes_refreshed: usize,
    /// IDs of nodes that were refreshed
    pub refreshed_node_ids: Vec<NodeId>,
    /// Time taken for refresh in milliseconds
    pub duration_ms: u64,
    /// Whether a full replay was required
    pub full_replay: bool,
}

impl RefreshResult {
    /// Creates a new refresh result
    pub fn new(nodes_refreshed: usize, refreshed_node_ids: Vec<NodeId>, duration_ms: u64) -> Self {
        Self {
            nodes_refreshed,
            refreshed_node_ids,
            duration_ms,
            full_replay: false,
        }
    }

    /// Creates a result for a full replay
    pub fn full_replay(nodes_refreshed: usize, duration_ms: u64) -> Self {
        Self {
            nodes_refreshed,
            refreshed_node_ids: Vec::new(),
            duration_ms,
            full_replay: true,
        }
    }
}

/// Handles incremental refresh of trace trees
pub struct IncrementalRefresher {
    /// State change detector
    detector: StateChangeDetector,
    /// Refresh strategy
    strategy: RefreshStrategy,
}

impl IncrementalRefresher {
    /// Creates a new incremental refresher
    pub fn new(strategy: RefreshStrategy) -> Self {
        Self {
            detector: StateChangeDetector::new(),
            strategy,
        }
    }

    /// Creates a refresher with an initial state
    pub fn with_state(state: ContractState, strategy: RefreshStrategy) -> Self {
        Self {
            detector: StateChangeDetector::with_state(state),
            strategy,
        }
    }

    /// Sets the refresh strategy
    pub fn set_strategy(&mut self, strategy: RefreshStrategy) {
        self.strategy = strategy;
    }

    /// Gets the current refresh strategy
    pub fn strategy(&self) -> RefreshStrategy {
        self.strategy
    }

    /// Performs an incremental refresh based on state changes
    pub fn refresh(
        &mut self,
        tree: &mut TraceTree,
        new_state: &ContractState,
    ) -> RefreshResult {
        let start_time = std::time::Instant::now();

        // Detect state changes
        let changes = self.detector.detect_changes(new_state);

        if changes.is_empty() {
            return RefreshResult::new(0, Vec::new(), start_time.elapsed().as_millis() as u64);
        }

        // Check if full replay is needed
        if self.requires_full_replay(&changes) {
            return self.perform_full_replay(tree, start_time);
        }

        // Perform incremental refresh
        self.perform_incremental_refresh(tree, &changes, start_time)
    }

    /// Checks if changes require a full replay
    fn requires_full_replay(&self, changes: &[StateChange]) -> bool {
        match self.strategy {
            RefreshStrategy::Full => true,
            _ => {
                // Full replay needed if code changed
                changes.iter().any(|c| c.is_code_change())
            }
        }
    }

    /// Performs a full replay of the trace
    fn perform_full_replay(
        &self,
        tree: &mut TraceTree,
        start_time: std::time::Instant,
    ) -> RefreshResult {
        let node_count = tree.node_count();
        
        // Mark all nodes for refresh
        for node_id in tree.node_ids() {
            if let Some(node) = tree.get_node_mut(node_id) {
                node.mark_for_refresh();
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;
        RefreshResult::full_replay(node_count, duration)
    }

    /// Performs an incremental refresh for specific changes
    fn perform_incremental_refresh(
        &self,
        tree: &mut TraceTree,
        changes: &[StateChange],
        start_time: std::time::Instant,
    ) -> RefreshResult {
        // Extract affected keys
        let affected_keys: Vec<Vec<u8>> = changes
            .iter()
            .map(|c| c.affected_key().to_vec())
            .collect();

        let code_changed = changes.iter().any(|c| c.is_code_change());

        // Mark affected nodes
        let mut affected_nodes = tree.mark_affected_nodes(&affected_keys, code_changed);

        // Apply strategy to expand the refresh set
        match self.strategy {
            RefreshStrategy::Minimal => {
                // Already have minimal set
            }
            RefreshStrategy::WithChildren => {
                affected_nodes = self.expand_with_children(tree, &affected_nodes);
            }
            RefreshStrategy::WithDescendants => {
                affected_nodes = self.expand_with_descendants(tree, &affected_nodes);
            }
            RefreshStrategy::Full => {
                // Already handled in requires_full_replay
            }
        }

        let nodes_refreshed = affected_nodes.len();
        let duration = start_time.elapsed().as_millis() as u64;

        RefreshResult::new(nodes_refreshed, affected_nodes, duration)
    }

    /// Expands the refresh set to include immediate children
    fn expand_with_children(&self, tree: &TraceTree, nodes: &[NodeId]) -> Vec<NodeId> {
        let mut expanded = HashSet::new();
        expanded.extend(nodes.iter());

        for &node_id in nodes {
            if let Some(node) = tree.get_node(node_id) {
                for &child_id in &node.children {
                    expanded.insert(child_id);
                }
            }
        }

        expanded.into_iter().collect()
    }

    /// Expands the refresh set to include all descendants
    fn expand_with_descendants(&self, tree: &TraceTree, nodes: &[NodeId]) -> Vec<NodeId> {
        let mut expanded = HashSet::new();
        let mut to_visit: Vec<NodeId> = nodes.to_vec();

        while let Some(node_id) = to_visit.pop() {
            if expanded.insert(node_id) {
                if let Some(node) = tree.get_node(node_id) {
                    to_visit.extend(&node.children);
                }
            }
        }

        expanded.into_iter().collect()
    }

    /// Resets the refresher with a new state
    pub fn reset(&mut self, state: ContractState) {
        self.detector.reset(state);
    }

    /// Clears the state detector
    pub fn clear(&mut self) {
        self.detector.clear();
    }
}

impl Default for IncrementalRefresher {
    fn default() -> Self {
        Self::new(RefreshStrategy::Minimal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ContractState;
    use crate::trace::TraceNode;

    fn create_test_tree() -> TraceTree {
        let mut tree = TraceTree::new();
        
        // Create a simple tree structure
        let mut root = TraceNode::new(0, "invoke".to_string());
        root.add_accessed_key(vec![1, 2]);
        let root_id = tree.add_node(root);

        let mut child1 = TraceNode::new(1, "read".to_string());
        child1.parent = Some(root_id);
        child1.add_accessed_key(vec![1, 2]);
        let child1_id = tree.add_node(child1);

        let mut child2 = TraceNode::new(2, "write".to_string());
        child2.parent = Some(root_id);
        child2.add_accessed_key(vec![3, 4]);
        let child2_id = tree.add_node(child2);

        if let Some(root) = tree.get_node_mut(root_id) {
            root.add_child(child1_id);
            root.add_child(child2_id);
        }

        tree
    }

    #[test]
    fn test_refresh_with_no_changes() {
        let mut refresher = IncrementalRefresher::new(RefreshStrategy::Minimal);
        let mut tree = create_test_tree();
        let state = ContractState::new(vec![1], "hash".to_string());

        let result = refresher.refresh(&mut tree, &state);
        assert_eq!(result.nodes_refreshed, 0);
        assert!(!result.full_replay);
    }

    #[test]
    fn test_refresh_with_ledger_change() {
        let mut refresher = IncrementalRefresher::new(RefreshStrategy::Minimal);
        let mut tree = create_test_tree();
        
        let mut state1 = ContractState::new(vec![1], "hash".to_string());
        state1.set_ledger_entry(vec![1, 2], b"value1");
        refresher.refresh(&mut tree, &state1);

        state1.set_ledger_entry(vec![1, 2], b"value2");
        let result = refresher.refresh(&mut tree, &state1);

        assert!(result.nodes_refreshed > 0);
        assert!(!result.full_replay);
    }

    #[test]
    fn test_refresh_with_code_change_requires_full_replay() {
        let mut refresher = IncrementalRefresher::new(RefreshStrategy::Minimal);
        let mut tree = create_test_tree();
        
        let mut state = ContractState::new(vec![1], "hash1".to_string());
        refresher.refresh(&mut tree, &state);

        state.set_code_hash(b"new_code");
        let result = refresher.refresh(&mut tree, &state);

        assert!(result.full_replay);
    }

    #[test]
    fn test_strategy_minimal() {
        let mut refresher = IncrementalRefresher::new(RefreshStrategy::Minimal);
        let mut tree = create_test_tree();
        
        let mut state = ContractState::new(vec![1], "hash".to_string());
        state.set_ledger_entry(vec![1, 2], b"value1");
        refresher.refresh(&mut tree, &state);

        state.set_ledger_entry(vec![1, 2], b"value2");
        let result = refresher.refresh(&mut tree, &state);

        // Should only refresh nodes that directly access the changed key
        assert!(result.nodes_refreshed <= 2);
    }

    #[test]
    fn test_strategy_with_children() {
        let mut refresher = IncrementalRefresher::new(RefreshStrategy::WithChildren);
        let mut tree = create_test_tree();
        
        let mut state = ContractState::new(vec![1], "hash".to_string());
        state.set_ledger_entry(vec![1, 2], b"value1");
        refresher.refresh(&mut tree, &state);

        state.set_ledger_entry(vec![1, 2], b"value2");
        let result = refresher.refresh(&mut tree, &state);

        // Should refresh affected nodes and their children
        assert!(result.nodes_refreshed >= 2);
    }

    #[test]
    fn test_strategy_full() {
        let mut refresher = IncrementalRefresher::new(RefreshStrategy::Full);
        let mut tree = create_test_tree();
        
        let mut state = ContractState::new(vec![1], "hash".to_string());
        state.set_ledger_entry(vec![1, 2], b"value1");
        refresher.refresh(&mut tree, &state);

        state.set_ledger_entry(vec![1, 2], b"value2");
        let result = refresher.refresh(&mut tree, &state);

        assert!(result.full_replay);
        assert_eq!(result.nodes_refreshed, tree.node_count());
    }

    #[test]
    fn test_set_strategy() {
        let mut refresher = IncrementalRefresher::new(RefreshStrategy::Minimal);
        assert_eq!(refresher.strategy(), RefreshStrategy::Minimal);

        refresher.set_strategy(RefreshStrategy::Full);
        assert_eq!(refresher.strategy(), RefreshStrategy::Full);
    }

    #[test]
    fn test_reset_and_clear() {
        let mut refresher = IncrementalRefresher::new(RefreshStrategy::Minimal);
        let state = ContractState::new(vec![1], "hash".to_string());

        refresher.reset(state);
        refresher.clear();
    }
}
