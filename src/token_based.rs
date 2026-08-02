//! Token-based distance algorithms (Person 2 assignment)
//!
//! Implements: Jaccard, Sorensen, Tversky, Overlap, Cosine, Tanimoto, Bag, MongeElkan

use std::collections::HashMap;
use std::hash::Hash;

use crate::base::{
    Base, count_counters, get_counter, intersect_counters, quick_answer_similarity, union_counters,
};
use crate::utils::find_ngrams;

// ─── helpers ────────────────────────────────────────────────────────────────

/// Build a frequency-counter for a generic slice tokenised by `qval`.
///
/// * `qval == 0` – not meaningful here; callers should split by words before calling.
/// * `qval == 1` – each element becomes its own single-element key.
/// * `qval > 1`  – sliding n-gram windows.
pub fn get_sequence_counter<T: Hash + Eq + Clone>(
    seq: &[T],
    qval: usize,
) -> HashMap<Vec<T>, usize> {
    if qval <= 1 {
        let items: Vec<Vec<T>> = seq.iter().map(|item| vec![item.clone()]).collect();
        get_counter(&items)
    } else {
        let ngrams = find_ngrams(seq, qval);
        get_counter(&ngrams)
    }
}

// ─── Jaccard ─────────────────────────────────────────────────────────────────

/// Jaccard similarity coefficient: |A ∩ B| / |A ∪ B|.
///
/// * `qval`   – tokenisation granularity (default 1 = per character).
/// * `as_set` – if true, only unique elements are counted.
///
/// <https://en.wikipedia.org/wiki/Jaccard_index>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jaccard {
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Jaccard {
    fn default() -> Self {
        Self {
            qval: 1,
            as_set: false,
        }
    }
}

impl Jaccard {
    pub fn new(qval: usize, as_set: bool) -> Self {
        Self { qval, as_set }
    }
}

impl<T: Hash + Eq + Clone + PartialEq> Base<T> for Jaccard {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if let Some(ans) = quick_answer_similarity(s1, s2, 1.0) {
            return ans;
        }

        let c1 = get_sequence_counter(s1, self.qval);
        let c2 = get_sequence_counter(s2, self.qval);

        let intersection = intersect_counters(&c1, &c2);
        let inter_cnt = count_counters(&intersection, self.as_set) as f64;

        let union = union_counters(&c1, &c2);
        let union_cnt = count_counters(&union, self.as_set) as f64;

        if union_cnt == 0.0 {
            1.0
        } else {
            inter_cnt / union_cnt
        }
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}

// ─── Sorensen ────────────────────────────────────────────────────────────────

/// Sørensen–Dice coefficient: 2·|A ∩ B| / (|A| + |B|).
///
/// <https://en.wikipedia.org/wiki/S%C3%B8rensen%E2%80%93Dice_coefficient>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sorensen {
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Sorensen {
    fn default() -> Self {
        Self {
            qval: 1,
            as_set: false,
        }
    }
}

impl Sorensen {
    pub fn new(qval: usize, as_set: bool) -> Self {
        Self { qval, as_set }
    }
}

impl<T: Hash + Eq + Clone + PartialEq> Base<T> for Sorensen {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if let Some(ans) = quick_answer_similarity(s1, s2, 1.0) {
            return ans;
        }

        let c1 = get_sequence_counter(s1, self.qval);
        let c2 = get_sequence_counter(s2, self.qval);

        let count =
            count_counters(&c1, self.as_set) as f64 + count_counters(&c2, self.as_set) as f64;
        let intersection = intersect_counters(&c1, &c2);
        let inter_cnt = count_counters(&intersection, self.as_set) as f64;

        2.0 * inter_cnt / count
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}

// ─── Overlap ─────────────────────────────────────────────────────────────────

/// Overlap (Szymkiewicz–Simpson) coefficient: |A ∩ B| / min(|A|, |B|).
///
/// <https://en.wikipedia.org/wiki/Overlap_coefficient>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overlap {
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Overlap {
    fn default() -> Self {
        Self {
            qval: 1,
            as_set: false,
        }
    }
}

impl Overlap {
    pub fn new(qval: usize, as_set: bool) -> Self {
        Self { qval, as_set }
    }
}

impl<T: Hash + Eq + Clone + PartialEq> Base<T> for Overlap {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if let Some(ans) = quick_answer_similarity(s1, s2, 1.0) {
            return ans;
        }

        let c1 = get_sequence_counter(s1, self.qval);
        let c2 = get_sequence_counter(s2, self.qval);

        let intersection = intersect_counters(&c1, &c2);
        let inter_cnt = count_counters(&intersection, self.as_set) as f64;

        let cnt1 = count_counters(&c1, self.as_set) as f64;
        let cnt2 = count_counters(&c2, self.as_set) as f64;
        let min_cnt = cnt1.min(cnt2);

        if min_cnt == 0.0 {
            0.0
        } else {
            inter_cnt / min_cnt
        }
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}

// ─── Cosine ──────────────────────────────────────────────────────────────────

/// Cosine similarity (Ochiai coefficient): |A ∩ B| / (|A| · |B|)^(1/N).
///
/// <https://en.wikipedia.org/wiki/Cosine_similarity>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cosine {
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Cosine {
    fn default() -> Self {
        Self {
            qval: 1,
            as_set: false,
        }
    }
}

impl Cosine {
    pub fn new(qval: usize, as_set: bool) -> Self {
        Self { qval, as_set }
    }
}

impl<T: Hash + Eq + Clone + PartialEq> Base<T> for Cosine {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if let Some(ans) = quick_answer_similarity(s1, s2, 1.0) {
            return ans;
        }

        let c1 = get_sequence_counter(s1, self.qval);
        let c2 = get_sequence_counter(s2, self.qval);

        let intersection = intersect_counters(&c1, &c2);
        let inter_cnt = count_counters(&intersection, self.as_set) as f64;

        let cnt1 = count_counters(&c1, self.as_set) as f64;
        let cnt2 = count_counters(&c2, self.as_set) as f64;

        // product of all sequence counts, then take N-th root (N = 2 sequences)
        let prod = cnt1 * cnt2;
        if prod == 0.0 {
            0.0
        } else {
            inter_cnt / prod.powf(1.0 / 2.0)
        }
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}

// ─── Tversky ─────────────────────────────────────────────────────────────────

/// Tversky index: |A ∩ B| / (|A ∩ B| + α·|A\B| + β·|B\A|).
///
/// Special cases:
/// * `ks = [1.0, 1.0]` → equivalent to Jaccard
/// * `ks = [0.5, 0.5]` → equivalent to Sørensen–Dice
///
/// When `bias` is `Some`, a two-sequence specialised formula is used.
///
/// <https://en.wikipedia.org/wiki/Tversky_index>
#[derive(Debug, Clone)]
pub struct Tversky {
    pub qval: usize,
    /// Weights [α, β] for the asymmetric terms.  Default `[1.0, 1.0]`.
    pub ks: Vec<f64>,
    /// Optional bias parameter activating the bias-corrected two-sequence formula.
    pub bias: Option<f64>,
    pub as_set: bool,
}

impl Default for Tversky {
    fn default() -> Self {
        Self {
            qval: 1,
            ks: vec![1.0, 1.0],
            bias: None,
            as_set: false,
        }
    }
}

impl Tversky {
    pub fn new(qval: usize, ks: Vec<f64>, bias: Option<f64>, as_set: bool) -> Self {
        Self {
            qval,
            ks,
            bias,
            as_set,
        }
    }
}

impl<T: Hash + Eq + Clone + PartialEq> Base<T> for Tversky {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if let Some(ans) = quick_answer_similarity(s1, s2, 1.0) {
            return ans;
        }

        let c1 = get_sequence_counter(s1, self.qval);
        let c2 = get_sequence_counter(s2, self.qval);

        let intersection = intersect_counters(&c1, &c2);
        let inter_cnt = count_counters(&intersection, self.as_set) as f64;

        let seq_counts = [
            count_counters(&c1, self.as_set) as f64,
            count_counters(&c2, self.as_set) as f64,
        ];

        // ks has exactly 2 weights for the 2-sequence case
        let ks: Vec<f64> = self
            .ks
            .iter()
            .copied()
            .chain(std::iter::repeat(1.0))
            .take(seq_counts.len())
            .collect();

        if seq_counts.len() != 2 || self.bias.is_none() {
            // General formula
            let mut result = inter_cnt;
            for (k, &s) in ks.iter().zip(seq_counts.iter()) {
                result += k * (s - inter_cnt);
            }
            if result == 0.0 {
                1.0
            } else {
                inter_cnt / result
            }
        } else if let Some(bias) = self.bias {
            // Bias-corrected two-sequence formula
            let (s1_cnt, s2_cnt) = (seq_counts[0], seq_counts[1]);
            let (alpha, beta) = (ks[0], ks[1]);
            let a_val = s1_cnt.min(s2_cnt);
            let b_val = s1_cnt.max(s2_cnt);
            let c_val = inter_cnt + bias;
            let result = alpha * beta * (a_val - b_val) + b_val * beta;
            c_val / (result + c_val)
        } else {
            unreachable!()
        }
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}

// ─── Tanimoto ────────────────────────────────────────────────────────────────

/// Tanimoto distance: log₂(Jaccard similarity), or -∞ when similarity is 0.
///
/// This is a logarithmic transform of the Jaccard index.
///
/// <https://en.wikipedia.org/wiki/Jaccard_index#Tanimoto_similarity_and_distance>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tanimoto {
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Tanimoto {
    fn default() -> Self {
        Self {
            qval: 1,
            as_set: false,
        }
    }
}

impl Tanimoto {
    pub fn new(qval: usize, as_set: bool) -> Self {
        Self { qval, as_set }
    }

    fn jaccard_similarity<T: Hash + Eq + Clone + PartialEq>(&self, s1: &[T], s2: &[T]) -> f64 {
        let inner = Jaccard {
            qval: self.qval,
            as_set: self.as_set,
        };
        inner.similarity(s1, s2)
    }
}

impl<T: Hash + Eq + Clone + PartialEq> Base<T> for Tanimoto {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        let jaccard = self.jaccard_similarity(s1, s2);
        if jaccard == 0.0 {
            f64::NEG_INFINITY
        } else {
            jaccard.log2()
        }
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}

// ─── Bag ─────────────────────────────────────────────────────────────────────

/// Bag distance: max(|A \ (A ∩ B)|, |B \ (A ∩ B)|).
///
/// Distance metric (not similarity-first).  `maximum` = max(|A|, |B|).
///
/// <http://www-db.disi.unibo.it/research/papers/SPIRE02.pdf>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bag {
    pub qval: usize,
}

impl Bag {
    pub fn new(qval: usize) -> Self {
        Self { qval }
    }
}

impl<T: Hash + Eq + Clone + PartialEq> Base<T> for Bag {
    /// Bag distance is a distance metric (lower = more similar).
    fn distance(&self, s1: &[T], s2: &[T]) -> f64 {
        if s1 == s2 {
            return 0.0;
        }

        let c1 = get_sequence_counter(s1, self.qval);
        let c2 = get_sequence_counter(s2, self.qval);

        let intersection = intersect_counters(&c1, &c2);

        // subtract intersection from each counter element-wise
        let remainder1: f64 = c1
            .iter()
            .map(|(k, &v)| {
                let inter_v = intersection.get(k).copied().unwrap_or(0);
                v.saturating_sub(inter_v) as f64
            })
            .sum();

        let remainder2: f64 = c2
            .iter()
            .map(|(k, &v)| {
                let inter_v = intersection.get(k).copied().unwrap_or(0);
                v.saturating_sub(inter_v) as f64
            })
            .sum();

        remainder1.max(remainder2)
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }
}

// ─── MongeElkan (TODO) ───────────────────────────────────────────────────────

// TODO: implement MongeElkan after DamerauLevenshtein is available from Person 1's edit_based.rs.
// MongeElkan imports DamerauLevenshtein from crate::edit_based.

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    // ── Jaccard ──────────────────────────────────────────────────────────────

    #[test]
    fn test_jaccard_python_values() {
        let alg = Jaccard::default();
        assert!((alg.similarity(&chars("test"), &chars("text")) - 3.0 / 5.0).abs() < 1e-9);
        assert!((alg.similarity(&chars("nelson"), &chars("neilsen")) - 5.0 / 8.0).abs() < 1e-9);
        assert!((alg.similarity(&chars("decide"), &chars("resize")) - 3.0 / 9.0).abs() < 1e-9);
    }

    #[test]
    fn test_jaccard_empty_and_identical() {
        let alg = Jaccard::default();
        let empty: Vec<char> = vec![];
        let s = chars("test");
        assert_eq!(alg.similarity(&empty, &empty), 1.0);
        assert_eq!(alg.similarity(&s, &s), 1.0);
        assert_eq!(alg.similarity(&s, &empty), 0.0);
        assert_eq!(alg.similarity(&empty, &s), 0.0);
    }

    #[test]
    fn test_jaccard_normalized() {
        let alg = Jaccard::default();
        let s1 = chars("test");
        let s2 = chars("text");
        let sim = alg.similarity(&s1, &s2);
        let dist = alg.distance(&s1, &s2);
        assert!((sim + dist - 1.0).abs() < 1e-9);
    }

    // ── Sorensen ─────────────────────────────────────────────────────────────

    #[test]
    fn test_sorensen_python_values() {
        let alg = Sorensen::default();
        // 2 * 3 / (4 + 4) = 6/8 = 0.75
        assert!((alg.similarity(&chars("test"), &chars("text")) - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_sorensen_empty_and_identical() {
        let alg = Sorensen::default();
        let empty: Vec<char> = vec![];
        let s = chars("test");
        assert_eq!(alg.similarity(&empty, &empty), 1.0);
        assert_eq!(alg.similarity(&s, &s), 1.0);
        assert_eq!(alg.similarity(&s, &empty), 0.0);
    }

    // ── Overlap ──────────────────────────────────────────────────────────────

    #[test]
    fn test_overlap_python_values() {
        let alg = Overlap::default();
        assert!((alg.similarity(&chars("test"), &chars("text")) - 3.0 / 4.0).abs() < 1e-9);
        // testme/textthis: min(6,8)=6, intersection=4 → 4/6
        assert!((alg.similarity(&chars("testme"), &chars("textthis")) - 4.0 / 6.0).abs() < 1e-6);
        // nelson/neilsen: min(6,7)=6, intersection=5 → 5/6
        assert!((alg.similarity(&chars("nelson"), &chars("neilsen")) - 5.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_overlap_empty() {
        let alg = Overlap::default();
        let empty: Vec<char> = vec![];
        assert_eq!(alg.similarity(&empty, &empty), 1.0);
        assert_eq!(alg.similarity(&chars("test"), &empty), 0.0);
    }

    // ── Cosine ───────────────────────────────────────────────────────────────

    #[test]
    fn test_cosine_python_values() {
        let alg = Cosine::default();
        // test/text: intersection={t:1,e:1,s:0,x:0...} = 3, counts 4,4 → 3/sqrt(16)=3/4=0.75
        assert!((alg.similarity(&chars("test"), &chars("text")) - 0.75).abs() < 1e-9);
        // nelson/neilsen: inter=5, counts 6,7 → 5/sqrt(42)
        let expected = 5.0 / (6.0_f64 * 7.0).sqrt();
        assert!((alg.similarity(&chars("nelson"), &chars("neilsen")) - expected).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_empty_and_identical() {
        let alg = Cosine::default();
        let empty: Vec<char> = vec![];
        let s = chars("test");
        assert_eq!(alg.similarity(&empty, &empty), 1.0);
        assert_eq!(alg.similarity(&s, &s), 1.0);
        assert_eq!(alg.similarity(&s, &empty), 0.0);
    }

    // ── Tversky ──────────────────────────────────────────────────────────────

    #[test]
    fn test_tversky_equals_jaccard() {
        // Tversky(ks=[1,1]) == Jaccard
        let jaccard = Jaccard::default();
        let tversky = Tversky::default(); // ks=[1.0, 1.0]
        for (a, b) in [
            ("test", "text"),
            ("nelson", "neilsen"),
            ("decide", "resize"),
        ] {
            let ja = jaccard.similarity(&chars(a), &chars(b));
            let tv = tversky.similarity(&chars(a), &chars(b));
            assert!(
                (ja - tv).abs() < 1e-9,
                "Jaccard={ja} Tversky={tv} for ({a},{b})"
            );
        }
    }

    #[test]
    fn test_tversky_equals_sorensen() {
        // Tversky(ks=[0.5,0.5]) == Sorensen
        let sorensen = Sorensen::default();
        let tversky = Tversky::new(1, vec![0.5, 0.5], None, false);
        for (a, b) in [("test", "text"), ("nelson", "neilsen")] {
            let so = sorensen.similarity(&chars(a), &chars(b));
            let tv = tversky.similarity(&chars(a), &chars(b));
            assert!(
                (so - tv).abs() < 1e-9,
                "Sorensen={so} Tversky={tv} for ({a},{b})"
            );
        }
    }

    #[test]
    fn test_tversky_empty_and_identical() {
        let alg = Tversky::default();
        let empty: Vec<char> = vec![];
        let s = chars("test");
        assert_eq!(alg.similarity(&empty, &empty), 1.0);
        assert_eq!(alg.similarity(&s, &s), 1.0);
        assert_eq!(alg.similarity(&s, &empty), 0.0);
    }

    // ── Tanimoto ─────────────────────────────────────────────────────────────

    #[test]
    fn test_tanimoto_identical() {
        let alg = Tanimoto::default();
        let s = chars("test");
        // Jaccard("test","test") = 1.0 → log2(1) = 0.0
        assert_eq!(alg.similarity(&s, &s), 0.0);
    }

    #[test]
    fn test_tanimoto_no_overlap() {
        let alg = Tanimoto::default();
        // Jaccard("abc","xyz") = 0 → -inf
        let s1 = chars("abc");
        let s2 = chars("xyz");
        assert_eq!(alg.similarity(&s1, &s2), f64::NEG_INFINITY);
    }

    #[test]
    fn test_tanimoto_partial_overlap() {
        let alg = Tanimoto::default();
        let s1 = chars("test");
        let s2 = chars("text");
        // Jaccard = 3/5 → log2(3/5) < 0
        let expected = (3.0_f64 / 5.0).log2();
        assert!((alg.similarity(&s1, &s2) - expected).abs() < 1e-9);
    }

    // ── Bag ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_bag_python_values() {
        let alg = Bag::default();
        assert_eq!(alg.distance(&chars("qwe"), &chars("qwe")), 0.0);
        assert_eq!(alg.distance(&chars("qwe"), &chars("erty")), 3.0);
        assert_eq!(alg.distance(&chars("qwe"), &chars("ewq")), 0.0); // anagram
        assert_eq!(alg.distance(&chars("qwe"), &chars("rtys")), 4.0);
    }

    #[test]
    fn test_bag_empty_and_identical() {
        let alg = Bag::default();
        let empty: Vec<char> = vec![];
        assert_eq!(alg.distance(&empty, &empty), 0.0);
        assert_eq!(alg.distance(&chars("test"), &chars("test")), 0.0);
    }

    #[test]
    fn test_bag_normalized() {
        let alg = Bag::default();
        let s1 = chars("test");
        let s2 = chars("text");
        let nd = alg.normalized_distance(&s1, &s2);
        let ns = alg.normalized_similarity(&s1, &s2);
        assert!((nd + ns - 1.0).abs() < 1e-9);
        assert!((0.0..=1.0).contains(&nd));
    }
}
