//! Interactive viewer for trace visualization and updates.
//!
//! This module provides the interactive viewer that displays trace output
//! and handles incremental updates when state changes occur.

use crate::refresh::RefreshResult;
use crate::trace::{NodeId, TraceNode, TraceTree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an update to the viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewerUpdate {
    /// A node was added
    NodeAdded { node_id: NodeId },
    /// A node was modified
    NodeModified { node_id: NodeId },
    /// A node was removed
    NodeRemoved { node_id: NodeId },
    /// Multiple nodes were refreshed
    NodesRefreshed { node_ids: Vec<NodeId> },
    /// The entire tree was refreshed
    FullRefresh,
}

/// Display format for trace nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayFormat {
    /// Compact single-line format
    Compact,
    /// Detailed multi-line format
    Detailed,
    /// JSON format
    Json,
}

/// Interactive viewer for trace visualization
pub struct InteractiveViewer {
    /// The trace tree being viewed
    tree: TraceTree,
    /// Display format
    format: DisplayFormat,
    /// Whether to show only changed nodes
    show_only_changed: bool,
    /// Update history
    update_history: Vec<ViewerUpdate>,
    /// Node display cache
    display_cache: HashMap<NodeId, String>,
}

impl InteractiveViewer {
    /// Creates a new interactive viewer
    pub fn new(tree: TraceTree) -> Self {
        Self {
            tree,
            format: DisplayFormat::Detailed,
            show_only_changed: false,
            update_history: Vec::new(),
            display_cache: HashMap::new(),
        }
    }

    /// Sets the display format
    pub fn set_format(&mut self, format: DisplayFormat) {
        self.format = format;
        self.invalidate_cache();
    }

    /// Gets the current display format
    pub fn format(&self) -> DisplayFormat {
        self.format
    }

    /// Sets whether to show only changed nodes
    pub fn set_show_only_changed(&mut self, show_only_changed: bool) {
        self.show_only_changed = show_only_changed;
    }

    /// Gets the trace tree
    pub fn tree(&self) -> &TraceTree {
        &self.tree
    }

    /// Gets a mutable reference to the trace tree
    pub fn tree_mut(&mut self) -> &mut TraceTree {
        &mut self.tree
    }

    /// Applies a refresh result to the viewer
    pub fn apply_refresh(&mut self, result: &RefreshResult) {
        if result.full_replay {
            self.update_history.push(ViewerUpdate::FullRefresh);
            self.invalidate_cache();
        } else {
            // Invalidate cache for refreshed nodes
            for &node_id in &result.refreshed_node_ids {
                self.display_cache.remove(&node_id);
            }

            if !result.refreshed_node_ids.is_empty() {
                self.update_history.push(ViewerUpdate::NodesRefreshed {
                    node_ids: result.refreshed_node_ids.clone(),
                });
            }
        }

        // Clear refresh flags after applying
        self.tree.clear_refresh_flags();
    }

    /// Renders the trace tree to a string
    pub fn render(&mut self) -> String {
        let mut output = String::new();

        if let Some(root_id) = self.tree.root() {
            self.render_node(root_id, 0, &mut output);
        } else {
            output.push_str("(empty trace)\n");
        }

        output
    }

    /// Renders a specific node and its children
    fn render_node(&mut self, node_id: NodeId, depth: usize, output: &mut String) {
        if let Some(node) = self.tree.get_node(node_id) {
            // Skip if showing only changed nodes and this node doesn't need refresh
            if self.show_only_changed && !node.needs_refresh {
                return;
            }

            // Check cache first
            let node_display = if let Some(cached) = self.display_cache.get(&node_id) {
                cached.clone()
            } else {
                let display = self.format_node(node, depth);
                self.display_cache.insert(node_id, display.clone());
                display
            };

            output.push_str(&node_display);

            // Render children
            let children = node.children.clone();
            for child_id in children {
                self.render_node(child_id, depth + 1, output);
            }
        }
    }

    /// Formats a single node for display
    fn format_node(&self, node: &TraceNode, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let refresh_marker = if node.needs_refresh { "*" } else { " " };

        match self.format {
            DisplayFormat::Compact => {
                format!(
                    "{}{} [{}] {}\n",
                    indent, refresh_marker, node.id, node.operation
                )
            }
            DisplayFormat::Detailed => {
                let mut output = format!(
                    "{}{} Node {} - {}\n",
                    indent, refresh_marker, node.id, node.operation
                );

                if !node.details.is_empty() {
                    for (key, value) in &node.details {
                        output.push_str(&format!("{}    {}: {}\n", indent, key, value));
                    }
                }

                if !node.accessed_keys.is_empty() {
                    output.push_str(&format!(
                        "{}    accessed_keys: {} key(s)\n",
                        indent,
                        node.accessed_keys.len()
                    ));
                }

                output
            }
            DisplayFormat::Json => {
                let json = serde_json::to_string_pretty(node).unwrap_or_default();
                format!("{}{}\n", indent, json.replace('\n', &format!("\n{}", indent)))
            }
        }
    }

    /// Gets the update history
    pub fn update_history(&self) -> &[ViewerUpdate] {
        &self.update_history
    }

    /// Clears the update history
    pub fn clear_history(&mut self) {
        self.update_history.clear();
    }

    /// Invalidates the entire display cache
    fn invalidate_cache(&mut self) {
        self.display_cache.clear();
    }

    /// Gets statistics about the current view
    pub fn statistics(&self) -> ViewerStatistics {
        let total_nodes = self.tree.node_count();
        let nodes_needing_refresh = self.tree.nodes_needing_refresh().len();
        let cached_nodes = self.display_cache.len();

        ViewerStatistics {
            total_nodes,
            nodes_needing_refresh,
            cached_nodes,
            update_count: self.update_history.len(),
        }
    }
}

/// Statistics about the viewer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerStatistics {
    /// Total number of nodes in the tree
    pub total_nodes: usize,
    /// Number of nodes that need refresh
    pub nodes_needing_refresh: usize,
    /// Number of nodes in the display cache
    pub cached_nodes: usize,
    /// Number of updates in history
    pub update_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> TraceTree {
        let mut tree = TraceTree::new();
        
        let mut root = TraceNode::new(0, "invoke".to_string());
        root.details.insert("contract".to_string(), "test".to_string());
        let root_id = tree.add_node(root);

        let mut child = TraceNode::new(1, "read".to_string());
        child.parent = Some(root_id);
        child.add_accessed_key(vec![1, 2, 3]);
        let child_id = tree.add_node(child);

        if let Some(root) = tree.get_node_mut(root_id) {
            root.add_child(child_id);
        }

        tree
    }

    #[test]
    fn test_viewer_creation() {
        let tree = create_test_tree();
        let viewer = InteractiveViewer::new(tree);
        
        assert_eq!(viewer.format(), DisplayFormat::Detailed);
        assert_eq!(viewer.tree().node_count(), 2);
    }

    #[test]
    fn test_set_format() {
        let tree = create_test_tree();
        let mut viewer = InteractiveViewer::new(tree);
        
        viewer.set_format(DisplayFormat::Compact);
        assert_eq!(viewer.format(), DisplayFormat::Compact);
        
        viewer.set_format(DisplayFormat::Json);
        assert_eq!(viewer.format(), DisplayFormat::Json);
    }

    #[test]
    fn test_render_empty_tree() {
        let tree = TraceTree::new();
        let mut viewer = InteractiveViewer::new(tree);
        
        let output = viewer.render();
        assert!(output.contains("empty trace"));
    }

    #[test]
    fn test_render_compact() {
        let tree = create_test_tree();
        let mut viewer = InteractiveViewer::new(tree);
        viewer.set_format(DisplayFormat::Compact);
        
        let output = viewer.render();
        assert!(output.contains("[0]"));
        assert!(output.contains("invoke"));
    }

    #[test]
    fn test_render_detailed() {
        let tree = create_test_tree();
        let mut viewer = InteractiveViewer::new(tree);
        viewer.set_format(DisplayFormat::Detailed);
        
        let output = viewer.render();
        assert!(output.contains("Node 0"));
        assert!(output.contains("invoke"));
        assert!(output.contains("contract: test"));
    }

    #[test]
    fn test_apply_refresh_full() {
        let tree = create_test_tree();
        let mut viewer = InteractiveViewer::new(tree);
        
        let result = RefreshResult::full_replay(2, 100);
        viewer.apply_refresh(&result);
        
        assert_eq!(viewer.update_history().len(), 1);
        assert!(matches!(
            viewer.update_history()[0],
            ViewerUpdate::FullRefresh
        ));
    }

    #[test]
    fn test_apply_refresh_incremental() {
        let tree = create_test_tree();
        let mut viewer = InteractiveViewer::new(tree);
        
        let result = RefreshResult::new(1, vec![0], 50);
        viewer.apply_refresh(&result);
        
        assert_eq!(viewer.update_history().len(), 1);
        assert!(matches!(
            viewer.update_history()[0],
            ViewerUpdate::NodesRefreshed { .. }
        ));
    }

    #[test]
    fn test_show_only_changed() {
        let mut tree = create_test_tree();
        
        // Mark one node for refresh
        if let Some(node) = tree.get_node_mut(0) {
            node.mark_for_refresh();
        }
        
        let mut viewer = InteractiveViewer::new(tree);
        viewer.set_show_only_changed(true);
        
        let output = viewer.render();
        // Should only show the marked node
        assert!(output.contains("Node 0"));
    }

    #[test]
    fn test_statistics() {
        let mut tree = create_test_tree();
        
        if let Some(node) = tree.get_node_mut(0) {
            node.mark_for_refresh();
        }
        
        let viewer = InteractiveViewer::new(tree);
        let stats = viewer.statistics();
        
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.nodes_needing_refresh, 1);
        assert_eq!(stats.update_count, 0);
    }

    #[test]
    fn test_clear_history() {
        let tree = create_test_tree();
        let mut viewer = InteractiveViewer::new(tree);
        
        let result = RefreshResult::new(1, vec![0], 50);
        viewer.apply_refresh(&result);
        assert_eq!(viewer.update_history().len(), 1);
        
        viewer.clear_history();
        assert_eq!(viewer.update_history().len(), 0);
    }
}
