//! State change detection for contract ledger entries and code artifacts.
//!
//! This module provides functionality to detect changes in contract state,
//! including ledger entries and code artifacts, enabling incremental trace refresh.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Represents a change in contract state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateChange {
    /// A ledger entry was modified
    LedgerEntryModified {
        key: Vec<u8>,
        old_hash: String,
        new_hash: String,
    },
    /// A ledger entry was added
    LedgerEntryAdded {
        key: Vec<u8>,
        hash: String,
    },
    /// A ledger entry was removed
    LedgerEntryRemoved {
        key: Vec<u8>,
        hash: String,
    },
    /// Contract code was updated
    CodeArtifactModified {
        contract_id: Vec<u8>,
        old_hash: String,
        new_hash: String,
    },
}

impl StateChange {
    /// Returns the affected key or contract ID
    pub fn affected_key(&self) -> &[u8] {
        match self {
            StateChange::LedgerEntryModified { key, .. } => key,
            StateChange::LedgerEntryAdded { key, .. } => key,
            StateChange::LedgerEntryRemoved { key, .. } => key,
            StateChange::CodeArtifactModified { contract_id, .. } => contract_id,
        }
    }

    /// Returns true if this is a code artifact change
    pub fn is_code_change(&self) -> bool {
        matches!(self, StateChange::CodeArtifactModified { .. })
    }
}

/// Represents the current state of a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractState {
    /// Contract identifier
    pub contract_id: Vec<u8>,
    /// Ledger entries with their hashes
    pub ledger_entries: HashMap<Vec<u8>, String>,
    /// Code artifact hash
    pub code_hash: String,
    /// Timestamp of last update
    pub last_updated: u64,
}

impl ContractState {
    /// Creates a new contract state
    pub fn new(contract_id: Vec<u8>, code_hash: String) -> Self {
        Self {
            contract_id,
            ledger_entries: HashMap::new(),
            code_hash,
            last_updated: 0,
        }
    }

    /// Adds or updates a ledger entry
    pub fn set_ledger_entry(&mut self, key: Vec<u8>, value: &[u8]) {
        let hash = Self::compute_hash(value);
        self.ledger_entries.insert(key, hash);
    }

    /// Removes a ledger entry
    pub fn remove_ledger_entry(&mut self, key: &[u8]) -> Option<String> {
        self.ledger_entries.remove(key)
    }

    /// Gets the hash of a ledger entry
    pub fn get_ledger_entry_hash(&self, key: &[u8]) -> Option<&String> {
        self.ledger_entries.get(key)
    }

    /// Updates the code hash
    pub fn set_code_hash(&mut self, code: &[u8]) {
        self.code_hash = Self::compute_hash(code);
    }

    /// Computes SHA-256 hash of data
    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}

/// Detects changes between contract states
pub struct StateChangeDetector {
    /// Previous state snapshot
    previous_state: Option<ContractState>,
}

impl StateChangeDetector {
    /// Creates a new state change detector
    pub fn new() -> Self {
        Self {
            previous_state: None,
        }
    }

    /// Creates a detector with an initial state
    pub fn with_state(state: ContractState) -> Self {
        Self {
            previous_state: Some(state),
        }
    }

    /// Detects changes between the previous state and a new state
    pub fn detect_changes(&mut self, new_state: &ContractState) -> Vec<StateChange> {
        let mut changes = Vec::new();

        if let Some(prev_state) = &self.previous_state {
            // Check for code changes
            if prev_state.code_hash != new_state.code_hash {
                changes.push(StateChange::CodeArtifactModified {
                    contract_id: new_state.contract_id.clone(),
                    old_hash: prev_state.code_hash.clone(),
                    new_hash: new_state.code_hash.clone(),
                });
            }

            // Check for ledger entry changes
            for (key, new_hash) in &new_state.ledger_entries {
                match prev_state.ledger_entries.get(key) {
                    Some(old_hash) if old_hash != new_hash => {
                        changes.push(StateChange::LedgerEntryModified {
                            key: key.clone(),
                            old_hash: old_hash.clone(),
                            new_hash: new_hash.clone(),
                        });
                    }
                    None => {
                        changes.push(StateChange::LedgerEntryAdded {
                            key: key.clone(),
                            hash: new_hash.clone(),
                        });
                    }
                    _ => {}
                }
            }

            // Check for removed entries
            for (key, hash) in &prev_state.ledger_entries {
                if !new_state.ledger_entries.contains_key(key) {
                    changes.push(StateChange::LedgerEntryRemoved {
                        key: key.clone(),
                        hash: hash.clone(),
                    });
                }
            }
        }

        // Update the previous state
        self.previous_state = Some(new_state.clone());

        changes
    }

    /// Resets the detector with a new state
    pub fn reset(&mut self, state: ContractState) {
        self.previous_state = Some(state);
    }

    /// Clears the previous state
    pub fn clear(&mut self) {
        self.previous_state = None;
    }
}

impl Default for StateChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_state_creation() {
        let contract_id = vec![1, 2, 3, 4];
        let code_hash = "abc123".to_string();
        let state = ContractState::new(contract_id.clone(), code_hash.clone());

        assert_eq!(state.contract_id, contract_id);
        assert_eq!(state.code_hash, code_hash);
        assert!(state.ledger_entries.is_empty());
    }

    #[test]
    fn test_ledger_entry_operations() {
        let mut state = ContractState::new(vec![1, 2, 3], "hash".to_string());
        let key = vec![10, 20];
        let value = b"test_value";

        state.set_ledger_entry(key.clone(), value);
        assert!(state.get_ledger_entry_hash(&key).is_some());

        let removed = state.remove_ledger_entry(&key);
        assert!(removed.is_some());
        assert!(state.get_ledger_entry_hash(&key).is_none());
    }

    #[test]
    fn test_detect_ledger_entry_added() {
        let mut detector = StateChangeDetector::new();
        let mut state1 = ContractState::new(vec![1], "hash".to_string());
        
        detector.detect_changes(&state1);

        state1.set_ledger_entry(vec![1, 2], b"value");
        let changes = detector.detect_changes(&state1);

        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], StateChange::LedgerEntryAdded { .. }));
    }

    #[test]
    fn test_detect_ledger_entry_modified() {
        let mut detector = StateChangeDetector::new();
        let mut state = ContractState::new(vec![1], "hash".to_string());
        state.set_ledger_entry(vec![1, 2], b"value1");
        
        detector.detect_changes(&state);

        state.set_ledger_entry(vec![1, 2], b"value2");
        let changes = detector.detect_changes(&state);

        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], StateChange::LedgerEntryModified { .. }));
    }

    #[test]
    fn test_detect_ledger_entry_removed() {
        let mut detector = StateChangeDetector::new();
        let mut state = ContractState::new(vec![1], "hash".to_string());
        state.set_ledger_entry(vec![1, 2], b"value");
        
        detector.detect_changes(&state);

        state.remove_ledger_entry(&vec![1, 2]);
        let changes = detector.detect_changes(&state);

        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], StateChange::LedgerEntryRemoved { .. }));
    }

    #[test]
    fn test_detect_code_artifact_modified() {
        let mut detector = StateChangeDetector::new();
        let mut state = ContractState::new(vec![1], "hash1".to_string());
        
        detector.detect_changes(&state);

        state.set_code_hash(b"new_code");
        let changes = detector.detect_changes(&state);

        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], StateChange::CodeArtifactModified { .. }));
        assert!(changes[0].is_code_change());
    }

    #[test]
    fn test_multiple_changes() {
        let mut detector = StateChangeDetector::new();
        let mut state = ContractState::new(vec![1], "hash1".to_string());
        state.set_ledger_entry(vec![1], b"value1");
        
        detector.detect_changes(&state);

        state.set_code_hash(b"new_code");
        state.set_ledger_entry(vec![1], b"value2");
        state.set_ledger_entry(vec![2], b"value3");
        
        let changes = detector.detect_changes(&state);

        assert_eq!(changes.len(), 3);
    }
}
