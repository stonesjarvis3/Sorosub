//! Trace tree structure and trace node management.
//!
//! This module defines the trace tree structure used to represent
//! contract execution traces and supports incremental updates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a trace node
pub type NodeId = usize;

/// Represents a segment of the trace that can be independently refreshed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSegment {
    /// Starting node ID
    pub start_node: NodeId,
    /// Ending node ID
    pub end_node: NodeId,
    /// State keys accessed in this segment
    pub accessed_keys: Vec<Vec<u8>>,
    /// Whether this segment depends on contract code
    pub depends_on_code: bool,
}

impl TraceSegment {
    /// Creates a new trace segment
    pub fn new(start_node: NodeId, end_node: NodeId) -> Self {
        Self {
            start_node,
            end_node,
            accessed_keys: Vec::new(),
            depends_on_code: false,
        }
    }

    /// Adds an accessed key to the segment
    pub fn add_accessed_key(&mut self, key: Vec<u8>) {
        if !self.accessed_keys.contains(&key) {
            self.accessed_keys.push(key);
        }
    }

    /// Marks the segment as depending on contract code
    pub fn mark_code_dependent(&mut self) {
        self.depends_on_code = true;
    }

    /// Checks if the segment is affected by a state key change
    pub fn is_affected_by_key(&self, key: &[u8]) -> bool {
        self.accessed_keys.iter().any(|k| k == key)
    }
}

/// Represents a node in the execution trace tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceNode {
    /// Unique node identifier
    pub id: NodeId,
    /// Parent node ID (None for root)
    pub parent: Option<NodeId>,
    /// Child node IDs
    pub children: Vec<NodeId>,
    /// Operation type (e.g., "invoke", "read", "write")
    pub operation: String,
    /// Operation details
    pub details: HashMap<String, String>,
    /// State keys accessed by this node
    pub accessed_keys: Vec<Vec<u8>>,
    /// Whether this node depends on contract code
    pub depends_on_code: bool,
    /// Timestamp of node creation
    pub timestamp: u64,
    /// Whether this node needs refresh
    pub needs_refresh: bool,
}

impl TraceNode {
    /// Creates a new trace node
    pub fn new(id: NodeId, operation: String) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            operation,
            details: HashMap::new(),
            accessed_keys: Vec::new(),
            depends_on_code: false,
            timestamp: 0,
            needs_refresh: false,
        }
    }

    /// Adds a child node
    pub fn add_child(&mut self, child_id: NodeId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Adds an accessed key
    pub fn add_accessed_key(&mut self, key: Vec<u8>) {
        if !self.accessed_keys.contains(&key) {
            self.accessed_keys.push(key);
        }
    }

    /// Marks the node as needing refresh
    pub fn mark_for_refresh(&mut self) {
        self.needs_refresh = true;
    }

    /// Clears the refresh flag
    pub fn clear_refresh_flag(&mut self) {
        self.needs_refresh = false;
    }

    /// Checks if the node is affected by a state key change
    pub fn is_affected_by_key(&self, key: &[u8]) -> bool {
        self.accessed_keys.iter().any(|k| k == key)
    }
}

/// Represents the complete execution trace tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceTree {
    /// All nodes in the tree
    nodes: HashMap<NodeId, TraceNode>,
    /// Root node ID
    root: Option<NodeId>,
    /// Next available node ID
    next_id: NodeId,
    /// Trace segments for incremental refresh
    segments: Vec<TraceSegment>,
}

impl TraceTree {
    /// Creates a new empty trace tree
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            next_id: 0,
            segments: Vec::new(),
        }
    }

    /// Adds a new node to the tree
    pub fn add_node(&mut self, mut node: TraceNode) -> NodeId {
        let id = self.next_id;
        node.id = id;
        self.next_id += 1;

        if self.root.is_none() {
            self.root = Some(id);
        }

        self.nodes.insert(id, node);
        id
    }

    /// Gets a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&TraceNode> {
        self.nodes.get(&id)
    }

    /// Gets a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut TraceNode> {
        self.nodes.get_mut(&id)
    }

    /// Gets the root node ID
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Adds a trace segment
    pub fn add_segment(&mut self, segment: TraceSegment) {
        self.segments.push(segment);
    }

    /// Gets all segments
    pub fn segments(&self) -> &[TraceSegment] {
        &self.segments
    }

    /// Marks nodes for refresh based on affected keys
    pub fn mark_affected_nodes(&mut self, affected_keys: &[Vec<u8>], code_changed: bool) -> Vec<NodeId> {
        let mut affected_nodes = Vec::new();

        for (id, node) in &mut self.nodes {
            let mut affected = false;

            if code_changed && node.depends_on_code {
                affected = true;
            }

            for key in affected_keys {
                if node.is_affected_by_key(key) {
                    affected = true;
                    break;
                }
            }

            if affected {
                node.mark_for_refresh();
                affected_nodes.push(*id);
            }
        }

        affected_nodes
    }

    /// Gets all nodes that need refresh
    pub fn nodes_needing_refresh(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.needs_refresh)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Clears refresh flags for all nodes
    pub fn clear_refresh_flags(&mut self) {
        for node in self.nodes.values_mut() {
            node.clear_refresh_flag();
        }
    }

    /// Gets the total number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Gets all node IDs
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }
}

impl Default for TraceTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_node_creation() {
        let node = TraceNode::new(0, "invoke".to_string());
        assert_eq!(node.id, 0);
        assert_eq!(node.operation, "invoke");
        assert!(node.children.is_empty());
        assert!(!node.needs_refresh);
    }

    #[test]
    fn test_trace_node_add_child() {
        let mut node = TraceNode::new(0, "invoke".to_string());
        node.add_child(1);
        node.add_child(2);
        assert_eq!(node.children.len(), 2);
        
        // Adding duplicate should not increase count
        node.add_child(1);
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn test_trace_node_accessed_keys() {
        let mut node = TraceNode::new(0, "read".to_string());
        let key1 = vec![1, 2, 3];
        let key2 = vec![4, 5, 6];

        node.add_accessed_key(key1.clone());
        node.add_accessed_key(key2.clone());

        assert!(node.is_affected_by_key(&key1));
        assert!(node.is_affected_by_key(&key2));
        assert!(!node.is_affected_by_key(&vec![7, 8, 9]));
    }

    #[test]
    fn test_trace_tree_add_node() {
        let mut tree = TraceTree::new();
        let node = TraceNode::new(0, "invoke".to_string());
        
        let id = tree.add_node(node);
        assert_eq!(id, 0);
        assert_eq!(tree.node_count(), 1);
        assert_eq!(tree.root(), Some(0));
    }

    #[test]
    fn test_trace_tree_mark_affected_nodes() {
        let mut tree = TraceTree::new();
        
        let mut node1 = TraceNode::new(0, "read".to_string());
        node1.add_accessed_key(vec![1, 2]);
        let id1 = tree.add_node(node1);

        let mut node2 = TraceNode::new(1, "write".to_string());
        node2.add_accessed_key(vec![3, 4]);
        let id2 = tree.add_node(node2);

        let affected = tree.mark_affected_nodes(&[vec![1, 2]], false);
        
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&id1));
        assert!(!affected.contains(&id2));
    }

    #[test]
    fn test_trace_tree_code_change_affects_nodes() {
        let mut tree = TraceTree::new();
        
        let mut node = TraceNode::new(0, "invoke".to_string());
        node.depends_on_code = true;
        tree.add_node(node);

        let affected = tree.mark_affected_nodes(&[], true);
        
        assert_eq!(affected.len(), 1);
    }

    #[test]
    fn test_trace_segment_creation() {
        let mut segment = TraceSegment::new(0, 5);
        assert_eq!(segment.start_node, 0);
        assert_eq!(segment.end_node, 5);
        assert!(!segment.depends_on_code);

        segment.add_accessed_key(vec![1, 2, 3]);
        assert!(segment.is_affected_by_key(&vec![1, 2, 3]));
        assert!(!segment.is_affected_by_key(&vec![4, 5, 6]));
    }

    #[test]
    fn test_clear_refresh_flags() {
        let mut tree = TraceTree::new();
        
        let mut node = TraceNode::new(0, "invoke".to_string());
        node.add_accessed_key(vec![1, 2]);
        tree.add_node(node);

        tree.mark_affected_nodes(&[vec![1, 2]], false);
        assert_eq!(tree.nodes_needing_refresh().len(), 1);

        tree.clear_refresh_flags();
        assert_eq!(tree.nodes_needing_refresh().len(), 0);
    }
}
