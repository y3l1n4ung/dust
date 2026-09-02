use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

/// Workspace analysis key for package-level feature flags.
pub const PACKAGE_FEATURES_ANALYSIS_KEY: &str = "dust_workspace.package_features.v1";

/// Package feature value used for Flutter packages.
pub const PACKAGE_FEATURE_FLUTTER: &str = "flutter";

/// One cached per-library analysis snapshot produced during the workspace scan phase.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LibraryAnalysisSnapshot {
    /// Per-key sorted string sets.
    string_sets: BTreeMap<String, Vec<String>>,
}

impl LibraryAnalysisSnapshot {
    /// Returns `true` when this snapshot contains no recorded analysis values.
    pub fn is_empty(&self) -> bool {
        self.string_sets.is_empty()
    }

    /// Returns the string-set values recorded for one analysis key.
    pub fn string_set(&self, key: &str) -> Option<&[String]> {
        self.string_sets.get(key).map(Vec::as_slice)
    }

    /// Returns all recorded string-set values keyed by analysis name.
    pub fn string_sets(&self) -> &BTreeMap<String, Vec<String>> {
        &self.string_sets
    }
}

/// The immutable workspace-wide analysis facts shared across all file emissions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceAnalysis {
    /// Immutable per-key workspace string sets.
    string_sets: BTreeMap<String, Arc<Vec<String>>>,
}

impl WorkspaceAnalysis {
    /// Returns the workspace-wide string-set values recorded for one analysis key.
    pub fn string_set(&self, key: &str) -> Option<&[String]> {
        self.string_sets.get(key).map(|values| values.as_slice())
    }

    /// Returns whether one string-set bucket contains `value`.
    pub fn contains_string_value(&self, key: &str, value: &str) -> bool {
        self.string_set(key)
            .is_some_and(|values| values.iter().any(|entry| entry == value))
    }

    /// Computes a deterministic FNV-1a hash of the entire workspace analysis.
    ///
    /// The hash is stable across builds because `BTreeMap` iterates in sorted
    /// key order and each value `Vec` is pre-sorted during `build()`.
    pub fn content_hash(&self) -> u64 {
        let mut hash = 1469598103934665603_u64;
        for (key, values) in &self.string_sets {
            for byte in key.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(1099511628211);
            }
            hash ^= 0;
            hash = hash.wrapping_mul(1099511628211);
            for value in values.iter() {
                for byte in value.as_bytes() {
                    hash ^= u64::from(*byte);
                    hash = hash.wrapping_mul(1099511628211);
                }
                hash ^= 0;
                hash = hash.wrapping_mul(1099511628211);
            }
        }
        hash
    }
}

/// A mutable builder used during the parse/scan phase to collect analysis facts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceAnalysisBuilder {
    /// Mutable per-key workspace string sets.
    string_sets: HashMap<String, HashSet<String>>,
}

impl WorkspaceAnalysisBuilder {
    /// Adds one value to a named string-set analysis bucket.
    pub fn add_string_set_value(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.string_sets
            .entry(key.into())
            .or_default()
            .insert(value.into());
    }

    /// Merges one cached per-library analysis snapshot into this builder.
    pub fn merge_snapshot(&mut self, snapshot: &LibraryAnalysisSnapshot) {
        for (key, values) in snapshot.string_sets() {
            let bucket = self.string_sets.entry(key.clone()).or_default();
            bucket.extend(values.iter().cloned());
        }
    }

    /// Merges another builder into this one.
    pub fn merge(&mut self, other: Self) {
        for (key, values) in other.string_sets {
            self.string_sets.entry(key).or_default().extend(values);
        }
    }

    /// Freezes this builder into one cached per-library snapshot.
    pub fn snapshot(&self) -> LibraryAnalysisSnapshot {
        let string_sets = self
            .string_sets
            .iter()
            .map(|(key, values)| {
                let mut values = values.iter().cloned().collect::<Vec<_>>();
                values.sort();
                (key.clone(), values)
            })
            .collect();
        LibraryAnalysisSnapshot { string_sets }
    }

    /// Freezes this builder into one immutable workspace-wide analysis set.
    pub fn build(self) -> WorkspaceAnalysis {
        let string_sets = self
            .string_sets
            .into_iter()
            .map(|(key, values)| {
                let mut values = values.into_iter().collect::<Vec<_>>();
                values.sort();
                (key, Arc::new(values))
            })
            .collect();
        WorkspaceAnalysis { string_sets }
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceAnalysisBuilder;

    #[test]
    fn snapshot_sorts_keys_and_values() {
        let mut builder = WorkspaceAnalysisBuilder::default();
        builder.add_string_set_value("b", "z");
        builder.add_string_set_value("a", "b");
        builder.add_string_set_value("a", "a");

        let snapshot = builder.snapshot();
        let keys = snapshot.string_sets().keys().cloned().collect::<Vec<_>>();
        assert_eq!(keys, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            snapshot.string_set("a"),
            Some(&["a".to_string(), "b".to_string()][..])
        );
    }

    #[test]
    fn content_hash_is_deterministic_regardless_of_insertion_order() {
        let mut builder_a = WorkspaceAnalysisBuilder::default();
        builder_a.add_string_set_value("z_key", "val_b");
        builder_a.add_string_set_value("a_key", "val_a");
        builder_a.add_string_set_value("z_key", "val_a");

        let mut builder_b = WorkspaceAnalysisBuilder::default();
        builder_b.add_string_set_value("a_key", "val_a");
        builder_b.add_string_set_value("z_key", "val_a");
        builder_b.add_string_set_value("z_key", "val_b");

        assert_eq!(
            builder_a.build().content_hash(),
            builder_b.build().content_hash()
        );
    }

    #[test]
    fn content_hash_changes_when_values_differ() {
        let mut builder_a = WorkspaceAnalysisBuilder::default();
        builder_a.add_string_set_value("key", "alpha");

        let mut builder_b = WorkspaceAnalysisBuilder::default();
        builder_b.add_string_set_value("key", "beta");

        assert_ne!(
            builder_a.build().content_hash(),
            builder_b.build().content_hash()
        );
    }

    #[test]
    fn content_hash_changes_when_key_added() {
        let mut builder_a = WorkspaceAnalysisBuilder::default();
        builder_a.add_string_set_value("key", "val");

        let mut builder_b = WorkspaceAnalysisBuilder::default();
        builder_b.add_string_set_value("key", "val");
        builder_b.add_string_set_value("extra", "val");

        assert_ne!(
            builder_a.build().content_hash(),
            builder_b.build().content_hash()
        );
    }

    #[test]
    fn build_sorts_values_for_deterministic_reads() {
        let mut builder = WorkspaceAnalysisBuilder::default();
        builder.add_string_set_value("copyable", "Gamma");
        builder.add_string_set_value("copyable", "Alpha");
        builder.add_string_set_value("copyable", "Beta");

        let analysis = builder.build();
        assert_eq!(
            analysis.string_set("copyable"),
            Some(&["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()][..])
        );
    }
}
