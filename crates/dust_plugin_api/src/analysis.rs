use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

/// One cached per-library analysis snapshot produced during the workspace scan phase.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LibraryAnalysisSnapshot {
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
    string_sets: BTreeMap<String, Arc<Vec<String>>>,
}

impl WorkspaceAnalysis {
    /// Returns the workspace-wide string-set values recorded for one analysis key.
    pub fn string_set(&self, key: &str) -> Option<&[String]> {
        self.string_sets.get(key).map(|values| values.as_slice())
    }
}

/// A mutable builder used during the parse/scan phase to collect analysis facts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceAnalysisBuilder {
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
