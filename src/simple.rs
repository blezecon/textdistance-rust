//! Simple distance and similarity metrics (Person 1 assignment).
//!
//! Ports the Python `textdistance.algorithms.simple` module: `Prefix`,
//! `Postfix`, `Length`, `Identity` and `Matrix`.

use std::collections::HashMap;
use std::hash::Hash;

use crate::base::{Base, is_ident};
use crate::utils::find_ngrams;

// ─── Prefix ──────────────────────────────────────────────────────────────────

/// Common prefix similarity.
///
/// The similarity is the length of the longest common prefix, measured over
/// the q-gram tokenization of the inputs (`qval`).
///
/// * `qval == 1` – compare elements directly (the default).
/// * `qval > 1`  – tokenize into q-grams before comparing.
/// * `qval == 0` – word splitting is a `str`-specific behavior and is not
///   supported for the generic `&[T]` API.
///
/// A custom element test `sim_test` can be supplied; when `None`, plain
/// equality is used. For `qval > 1`, q-grams are compared by equality.
#[derive(Debug, Clone, Copy)]
pub struct Prefix<T> {
    /// q-gram size; `1` treats the input as individual elements.
    pub qval: usize,
    /// Optional element equality test, defaulting to `PartialEq`.
    pub sim_test: Option<fn(&T, &T) -> bool>,
}

impl<T> Default for Prefix<T> {
    fn default() -> Self {
        Self {
            qval: 1,
            sim_test: None,
        }
    }
}

impl<T> Prefix<T> {
    /// Create a `Prefix` metric with the given q-gram size and element test.
    pub fn new(qval: usize, sim_test: Option<fn(&T, &T) -> bool>) -> Self {
        Self { qval, sim_test }
    }

    /// Compute the common prefix of two sequences as a sequence of elements.
    ///
    /// This mirrors the Python `__call__` result and is element-level
    /// (`qval <= 1`). For `qval > 1` use [`Base::similarity`].
    pub fn prefix(&self, s1: &[T], s2: &[T]) -> Vec<T>
    where
        T: Clone + PartialEq,
    {
        let test = self.sim_test.unwrap_or(|a, b| a == b);
        s1.iter()
            .zip(s2.iter())
            .take_while(|(a, b)| test(a, b))
            .map(|(a, _)| a.clone())
            .collect()
    }
}

impl<T: Clone + PartialEq> Base<T> for Prefix<T> {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            self.prefix(s1, s2).len() as f64
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            g1.iter().zip(g2.iter()).take_while(|(a, b)| a == b).count() as f64
        }
    }
}

// ─── Postfix ─────────────────────────────────────────────────────────────────

/// Common postfix similarity.
///
/// Delegates to [`Prefix`] after reversing both inputs. The Python reference
/// crashes for `qval > 1` (it tries to join q-gram tuples with `''`); this
/// implementation instead computes the common q-gram postfix length.
#[derive(Debug, Clone, Copy)]
pub struct Postfix<T> {
    inner: Prefix<T>,
}

impl<T> Default for Postfix<T> {
    fn default() -> Self {
        Self {
            inner: Prefix::default(),
        }
    }
}

impl<T> Postfix<T> {
    /// Create a `Postfix` metric with the given q-gram size and element test.
    pub fn new(qval: usize, sim_test: Option<fn(&T, &T) -> bool>) -> Self {
        Self {
            inner: Prefix::new(qval, sim_test),
        }
    }

    /// Compute the common postfix of two sequences as a sequence of elements.
    ///
    /// Element-level (`qval <= 1`); mirrors the Python `__call__` result.
    pub fn postfix(&self, s1: &[T], s2: &[T]) -> Vec<T>
    where
        T: Clone + PartialEq,
    {
        let rev1: Vec<T> = s1.iter().rev().cloned().collect();
        let rev2: Vec<T> = s2.iter().rev().cloned().collect();
        let mut result = self.inner.prefix(&rev1, &rev2);
        result.reverse();
        result
    }
}

impl<T: Clone + PartialEq> Base<T> for Postfix<T> {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        let rev1: Vec<T> = s1.iter().rev().cloned().collect();
        let rev2: Vec<T> = s2.iter().rev().cloned().collect();
        self.inner.similarity(&rev1, &rev2)
    }
}

// ─── Length ──────────────────────────────────────────────────────────────────

/// Length distance.
///
/// The distance is the absolute difference of the sequence lengths:
/// `max(len(s1), len(s2)) - min(len(s1), len(s2))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Length;

impl<T: PartialEq> Base<T> for Length {
    fn distance(&self, s1: &[T], s2: &[T]) -> f64 {
        let l1 = s1.len() as f64;
        let l2 = s2.len() as f64;
        l1.max(l2) - l1.min(l2)
    }
}

// ─── Identity ────────────────────────────────────────────────────────────────

/// Identity similarity.
///
/// Returns `1.0` when the sequences are identical and `0.0` otherwise.
/// The maximum is always `1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Identity;

impl<T: PartialEq> Base<T> for Identity {
    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if is_ident(s1, s2) { 1.0 } else { 0.0 }
    }
}

// ─── Matrix ──────────────────────────────────────────────────────────────────

/// Matrix similarity.
///
/// Looks up the similarity of two single-element sequences in a cost matrix.
/// When the pair is not found and `symmetric` is true, the transposed pair is
/// tried. Falls back to `match_cost` for identical inputs and to
/// `mismatch_cost` otherwise. With no matrix, only the match/mismatch costs
/// are used.
///
/// Note: the Python reference keys the matrix on tuples of whole sequences;
/// this port keys on element pairs `(T, T)`, matching the common single-char
/// usage.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T: Hash + Eq> {
    /// Explicit substitution matrix mapping `(a, b)` pairs to scores.
    pub mat: Option<HashMap<(T, T), f64>>,
    /// Score for aligning two mismatching elements.
    pub mismatch_cost: f64,
    /// Score for aligning two matching elements.
    pub match_cost: f64,
    /// Whether the matrix is symmetric (`mat(a, b) == mat(b, a)`).
    pub symmetric: bool,
}

impl<T: Hash + Eq> Default for Matrix<T> {
    fn default() -> Self {
        Self {
            mat: None,
            mismatch_cost: 0.0,
            match_cost: 1.0,
            symmetric: true,
        }
    }
}

impl<T: Hash + Eq> Matrix<T> {
    /// Create a `Matrix` metric.
    ///
    /// * `mat`             – optional `(T, T) -> f64` lookup table.
    /// * `mismatch_cost`   – cost returned when no entry matches.
    /// * `match_cost`      – cost returned for identical inputs (and the maximum).
    /// * `symmetric`       – also try the transposed pair on lookup.
    pub fn new(
        mat: Option<HashMap<(T, T), f64>>,
        mismatch_cost: f64,
        match_cost: f64,
        symmetric: bool,
    ) -> Self {
        Self {
            mat,
            mismatch_cost,
            match_cost,
            symmetric,
        }
    }
}

impl<T: Hash + Eq + Clone> Base<T> for Matrix<T> {
    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        self.match_cost
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        match &self.mat {
            None => {
                if is_ident(s1, s2) {
                    self.match_cost
                } else {
                    self.mismatch_cost
                }
            }
            Some(mat) => {
                if let (Some(a), Some(b)) = (s1.first(), s2.first()) {
                    let key = (a.clone(), b.clone());
                    if let Some(&cost) = mat.get(&key) {
                        return cost;
                    }
                    if self.symmetric
                        && let Some(&cost) = mat.get(&(key.1, key.0))
                    {
                        return cost;
                    }
                }
                if is_ident(s1, s2) {
                    self.match_cost
                } else {
                    self.mismatch_cost
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn test_prefix_similarity() {
        let p = Prefix::default();
        assert_eq!(p.similarity(&chars("abcdef"), &chars("abcz")), 3.0);
        assert_eq!(p.similarity(&chars(""), &chars("")), 0.0);
        assert_eq!(p.similarity(&chars("hello"), &chars("hello")), 5.0);
        assert_eq!(p.similarity(&chars("xyz"), &chars("abc")), 0.0);
        assert_eq!(p.similarity(&chars("héllo"), &chars("héllo!")), 5.0);
        assert_eq!(p.similarity(&chars("abc"), &chars("ab")), 2.0);
    }

    #[test]
    fn test_prefix_sequence() {
        let p = Prefix::default();
        assert_eq!(p.prefix(&chars("abcdef"), &chars("abcz")), chars("abc"));
        assert_eq!(p.prefix(&chars("xyz"), &chars("abc")), Vec::<char>::new());
        assert_eq!(p.prefix(&chars(""), &chars("")), Vec::<char>::new());
    }

    #[test]
    fn test_prefix_distance() {
        let p = Prefix::default();
        assert_eq!(p.distance(&chars("abcdef"), &chars("abcz")), 3.0);
        assert_eq!(p.normalized_distance(&chars("abcdef"), &chars("abcz")), 0.5);
        assert_eq!(
            p.normalized_similarity(&chars("abcdef"), &chars("abcz")),
            0.5
        );
    }

    #[test]
    fn test_prefix_qval() {
        let p = Prefix::new(2, None);
        assert_eq!(p.similarity(&chars("abcdef"), &chars("abcxyz")), 2.0);
        assert_eq!(p.similarity(&chars("ab"), &chars("ab")), 1.0);
    }

    #[test]
    fn test_prefix_sim_test() {
        fn case_insensitive(a: &char, b: &char) -> bool {
            a.to_lowercase().next() == b.to_lowercase().next()
        }
        let p = Prefix::new(1, Some(case_insensitive));
        assert_eq!(p.similarity(&chars("ABC"), &chars("abc")), 3.0);
        assert_eq!(p.prefix(&chars("ABC"), &chars("abc")), chars("ABC"));
    }

    #[test]
    fn test_postfix() {
        let q = Postfix::default();
        assert_eq!(q.similarity(&chars("abcdef"), &chars("xyzdef")), 3.0);
        assert_eq!(q.postfix(&chars("abcdef"), &chars("xyzdef")), chars("def"));
        assert_eq!(q.similarity(&chars("hello"), &chars("hello")), 5.0);
        assert_eq!(q.similarity(&chars(""), &chars("")), 0.0);
        assert_eq!(q.similarity(&chars("abc"), &chars("xyz")), 0.0);
    }

    #[test]
    fn test_length() {
        let l = Length;
        assert_eq!(l.distance(&chars("abc"), &chars("ab")), 1.0);
        assert_eq!(l.similarity(&chars("abc"), &chars("ab")), 2.0);
        assert_eq!(l.distance(&chars(""), &chars("")), 0.0);
        assert_eq!(l.distance(&chars("aa"), &chars("aa")), 0.0);
        assert_eq!(l.distance(&chars(""), &chars("qwertyui")), 8.0);
    }

    #[test]
    fn test_identity() {
        let i = Identity;
        assert_eq!(i.similarity(&chars("aa"), &chars("aa")), 1.0);
        assert_eq!(i.similarity(&chars("ab"), &chars("ab")), 1.0);
        assert_eq!(i.distance(&chars("ab"), &chars("ab")), 0.0);
        assert_eq!(i.similarity(&chars(""), &chars("")), 1.0);
        assert_eq!(i.similarity(&chars("a"), &chars("b")), 0.0);
        assert_eq!(i.distance(&chars("a"), &chars("b")), 1.0);
    }

    #[test]
    fn test_matrix_no_map() {
        let m = Matrix::<char>::default();
        assert_eq!(m.similarity(&chars("a"), &chars("b")), 0.0);
        assert_eq!(m.similarity(&chars("a"), &chars("a")), 1.0);
        assert_eq!(m.maximum(&chars("a"), &chars("b")), 1.0);
    }

    #[test]
    fn test_matrix_map() {
        let mut mat = HashMap::new();
        mat.insert(('a', 'a'), 1.0);
        mat.insert(('a', 'b'), 0.5);
        let m = Matrix::new(Some(mat), 0.0, 1.0, true);
        assert_eq!(m.similarity(&chars("a"), &chars("a")), 1.0);
        assert_eq!(m.similarity(&chars("a"), &chars("b")), 0.5);
        assert_eq!(m.similarity(&chars("b"), &chars("a")), 0.5);
        assert_eq!(m.similarity(&chars("c"), &chars("d")), 0.0);
    }

    #[test]
    fn test_matrix_non_symmetric() {
        let mut mat = HashMap::new();
        mat.insert(('a', 'b'), 0.25);
        let m = Matrix::new(Some(mat), 0.0, 1.0, false);
        assert_eq!(m.similarity(&chars("a"), &chars("b")), 0.25);
        assert_eq!(m.similarity(&chars("b"), &chars("a")), 0.0);
    }

    #[test]
    fn test_matrix_custom_costs() {
        let mut mat = HashMap::new();
        mat.insert(('a', 'b'), 0.9);
        let m = Matrix::new(Some(mat), 0.1, 2.0, true);
        assert_eq!(m.similarity(&chars("a"), &chars("b")), 0.9);
        assert_eq!(m.maximum(&chars("a"), &chars("b")), 2.0);
        assert_eq!(m.similarity(&chars("a"), &chars("a")), 2.0);
    }

    #[test]
    fn test_normalization_invariants() {
        let algs: Vec<Box<dyn Base<char>>> = vec![
            Box::new(Prefix::default()),
            Box::new(Postfix::default()),
            Box::new(Length),
            Box::new(Identity),
            Box::new(Matrix::<char>::default()),
        ];
        for alg in algs {
            let d = alg.normalized_distance(&chars("abcde"), &chars("abxde"));
            let s = alg.normalized_similarity(&chars("abcde"), &chars("abxde"));
            assert!((0.0..=1.0).contains(&d));
            assert!((0.0..=1.0).contains(&s));
            assert!((s + d - 1.0).abs() < 1e-9);
        }
    }
}
