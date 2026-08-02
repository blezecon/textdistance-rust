//! Core traits and counter utilities matching Python textdistance.algorithms.base

use std::collections::HashMap;
use std::hash::Hash;

/// Base trait for string distance and similarity metrics.
pub trait Base<T: PartialEq> {
    /// Compute raw distance between two sequences.
    fn distance(&self, s1: &[T], s2: &[T]) -> f64 {
        self.maximum(s1, s2) - self.similarity(s1, s2)
    }

    /// Compute raw similarity between two sequences.
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        self.maximum(s1, s2) - self.distance(s1, s2)
    }

    /// Return the maximum possible distance/similarity value for the given inputs.
    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }

    /// Compute normalized distance in range [0.0, 1.0].
    fn normalized_distance(&self, s1: &[T], s2: &[T]) -> f64 {
        let max_val = self.maximum(s1, s2);
        if max_val == 0.0 {
            0.0
        } else {
            self.distance(s1, s2) / max_val
        }
    }

    /// Compute normalized similarity in range [0.0, 1.0].
    fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

/// Helper function to check if two sequences are identical.
pub fn is_ident<T: PartialEq>(s1: &[T], s2: &[T]) -> bool {
    s1 == s2
}

/// Quick answer check for distance-first algorithms.
/// Returns Some(val) if a quick answer exists, or None.
pub fn quick_answer_distance<T: PartialEq>(s1: &[T], s2: &[T], max_val: f64) -> Option<f64> {
    if is_ident(s1, s2) {
        return Some(0.0);
    }
    if s1.is_empty() || s2.is_empty() {
        return Some(max_val);
    }
    None
}

/// Quick answer check for similarity-first algorithms.
/// Returns Some(val) if a quick answer exists, or None.
pub fn quick_answer_similarity<T: PartialEq>(s1: &[T], s2: &[T], max_val: f64) -> Option<f64> {
    if is_ident(s1, s2) {
        return Some(max_val);
    }
    if s1.is_empty() || s2.is_empty() {
        return Some(0.0);
    }
    None
}

/// Convert a sequence of items into a frequency counter map.
pub fn get_counter<T: Hash + Eq + Clone>(seq: &[T]) -> HashMap<T, usize> {
    let mut map = HashMap::new();
    for item in seq {
        *map.entry(item.clone()).or_insert(0) += 1;
    }
    map
}

/// Compute element-wise minimum intersection of two counters (Python Counter `&`).
pub fn intersect_counters<T: Hash + Eq + Clone>(
    c1: &HashMap<T, usize>,
    c2: &HashMap<T, usize>,
) -> HashMap<T, usize> {
    let mut result = HashMap::new();
    for (k, &v1) in c1 {
        if let Some(&v2) = c2.get(k) {
            let min_v = v1.min(v2);
            if min_v > 0 {
                result.insert(k.clone(), min_v);
            }
        }
    }
    result
}

/// Compute element-wise maximum union of two counters (Python Counter `|`).
pub fn union_counters<T: Hash + Eq + Clone>(
    c1: &HashMap<T, usize>,
    c2: &HashMap<T, usize>,
) -> HashMap<T, usize> {
    let mut result = c1.clone();
    for (k, &v2) in c2 {
        let entry = result.entry(k.clone()).or_insert(0);
        *entry = (*entry).max(v2);
    }
    result
}

/// Compute element-wise sum of two counters (Python Counter `+`).
pub fn sum_counters<T: Hash + Eq + Clone>(
    c1: &HashMap<T, usize>,
    c2: &HashMap<T, usize>,
) -> HashMap<T, usize> {
    let mut result = c1.clone();
    for (k, &v2) in c2 {
        *result.entry(k.clone()).or_insert(0) += v2;
    }
    result
}

/// Count elements in a counter, either total frequency or unique key count (as_set).
pub fn count_counters<T: Hash + Eq>(counter: &HashMap<T, usize>, as_set: bool) -> usize {
    if as_set {
        counter.len()
    } else {
        counter.values().sum()
    }
}
