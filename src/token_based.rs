//! Token-based distance algorithms (Person 2 assignment)
//!
//! Implements: Jaccard, Sorensen, Tversky, Overlap, Cosine, Tanimoto, Bag, MongeElkan

use std::collections::HashMap;
use std::hash::Hash;

use crate::base::{Base, get_counter, quick_answer_similarity};
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

/// Build a slice-borrowing frequency counter for `seq` tokenised by `qval`.
///
/// Borrows slices directly from `seq` as map keys (`&[T]`), eliminating heap key allocations.
pub fn get_slice_counter<T: Hash + Eq>(seq: &[T], qval: usize) -> HashMap<&[T], usize> {
    let mut map = HashMap::new();
    if seq.is_empty() {
        return map;
    }
    let n = seq.len();
    if qval <= 1 {
        for i in 0..n {
            *map.entry(&seq[i..i + 1]).or_insert(0) += 1;
        }
    } else if n >= qval {
        for i in 0..=n - qval {
            *map.entry(&seq[i..i + qval]).or_insert(0) += 1;
        }
    }
    map
}

/// Compute (c1_count, c2_count, intersection_count) from two slice counters without allocating.
pub fn compute_counts<T: Hash + Eq>(
    c1: &HashMap<&[T], usize>,
    c2: &HashMap<&[T], usize>,
    as_set: bool,
) -> (f64, f64, f64) {
    if as_set {
        let c1_cnt = c1.len() as f64;
        let c2_cnt = c2.len() as f64;
        let mut inter_cnt = 0f64;
        for k in c1.keys() {
            if c2.contains_key(k) {
                inter_cnt += 1.0;
            }
        }
        (c1_cnt, c2_cnt, inter_cnt)
    } else {
        let mut c1_cnt = 0usize;
        let mut inter_cnt = 0usize;
        for (k, &v1) in c1 {
            c1_cnt += v1;
            if let Some(&v2) = c2.get(k) {
                inter_cnt += v1.min(v2);
            }
        }
        let c2_cnt: usize = c2.values().sum();
        (c1_cnt as f64, c2_cnt as f64, inter_cnt as f64)
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

        let c1 = get_slice_counter(s1, self.qval);
        let c2 = get_slice_counter(s2, self.qval);

        let (c1_cnt, c2_cnt, inter_cnt) = compute_counts(&c1, &c2, self.as_set);
        let union_cnt = c1_cnt + c2_cnt - inter_cnt;

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

        let c1 = get_slice_counter(s1, self.qval);
        let c2 = get_slice_counter(s2, self.qval);

        let (c1_cnt, c2_cnt, inter_cnt) = compute_counts(&c1, &c2, self.as_set);
        let total_cnt = c1_cnt + c2_cnt;

        if total_cnt == 0.0 {
            1.0
        } else {
            2.0 * inter_cnt / total_cnt
        }
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

        let c1 = get_slice_counter(s1, self.qval);
        let c2 = get_slice_counter(s2, self.qval);

        let (c1_cnt, c2_cnt, inter_cnt) = compute_counts(&c1, &c2, self.as_set);
        let min_cnt = c1_cnt.min(c2_cnt);

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

        let c1 = get_slice_counter(s1, self.qval);
        let c2 = get_slice_counter(s2, self.qval);

        let (c1_cnt, c2_cnt, inter_cnt) = compute_counts(&c1, &c2, self.as_set);

        let prod = c1_cnt * c2_cnt;
        if prod == 0.0 {
            0.0
        } else {
            inter_cnt / prod.sqrt()
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

        let c1 = get_slice_counter(s1, self.qval);
        let c2 = get_slice_counter(s2, self.qval);

        let (s1_cnt, s2_cnt, inter_cnt) = compute_counts(&c1, &c2, self.as_set);
        let seq_counts = [s1_cnt, s2_cnt];

        let ks: Vec<f64> = self
            .ks
            .iter()
            .copied()
            .chain(std::iter::repeat(1.0))
            .take(2)
            .collect();

        if self.bias.is_none() {
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

        let c1 = get_slice_counter(s1, self.qval);
        let c2 = get_slice_counter(s2, self.qval);

        let (c1_cnt, c2_cnt, inter_cnt) = compute_counts(&c1, &c2, false);
        let remainder1 = c1_cnt - inter_cnt;
        let remainder2 = c2_cnt - inter_cnt;

        remainder1.max(remainder2)
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }
}

// ─── MongeElkan ──────────────────────────────────────────────────────────────

/// Monge-Elkan similarity between two token sequences.
///
/// For each token `c1` in `seq1`, find the maximum similarity against every token
/// `c2` in `seq2` using an inner algorithm (default: `DamerauLevenshtein`).
/// The score is the mean of those per-token maxima divided by `len(seq1)`.
///
/// When `symmetric = true`, the result is averaged over both orderings
/// (seq1→seq2 and seq2→seq1), matching Python's `permutations` behaviour.
///
/// The inner algorithm is `DamerauLevenshtein` with default parameters, matching
/// the Python `_damerau_levenshtein = DamerauLevenshtein()` class attribute.
///
/// <https://www.academia.edu/200314/Generalized_Monge-Elkan_Method_for_Approximate_Text_String_Comparison>
#[derive(Debug, Clone)]
pub struct MongeElkan {
    pub symmetric: bool,
    pub qval: usize,
}

impl Default for MongeElkan {
    fn default() -> Self {
        Self {
            symmetric: false,
            qval: 1,
        }
    }
}

impl MongeElkan {
    pub fn new(symmetric: bool, qval: usize) -> Self {
        Self { symmetric, qval }
    }

    /// Tokenise a string into a Vec of token-Vecs according to `qval`.
    ///
    /// * `qval == 0` / `qval == 1` → each character is its own token (`Vec<char>` of length 1).
    /// * `qval > 1`  → n-gram windows.
    fn tokenise(s: &str, qval: usize) -> Vec<Vec<char>> {
        let chars: Vec<char> = s.chars().collect();
        if qval <= 1 {
            chars.iter().map(|&c| vec![c]).collect()
        } else {
            find_ngrams(&chars, qval)
        }
    }

    /// Core Monge-Elkan calculation: `seq` is the "outer" sequence,
    /// `other` is the "inner" sequence we compare each token against.
    ///
    /// Uses `DamerauLevenshtein::default()` as inner algorithm, matching Python.
    fn calc(seq: &[Vec<char>], other: &[Vec<char>]) -> f64 {
        use crate::edit_based::DamerauLevenshtein;

        if seq.is_empty() {
            return 0.0;
        }

        let inner: DamerauLevenshtein<char> = DamerauLevenshtein::default();
        let mut maxes: Vec<f64> = Vec::new();

        for c1 in seq {
            let mut max_sim = f64::NEG_INFINITY;
            for c2 in other {
                let sim = inner.similarity(c1, c2);
                if sim > max_sim {
                    max_sim = sim;
                }
            }
            maxes.push(max_sim);
        }

        // Python: sum(maxes) / len(seq) / len(maxes)
        // len(maxes) == len(seq) * len(sequences-1), but for 2 seqs it simplifies
        // to sum(maxes) / len(seq) / len(seq)  (one comparison seq → each token maxed over other)
        // Actually: for _calc(seq, *sequences) with one extra arg, len(maxes) = len(seq)*1
        // so: sum / len(seq) / len(seq) => sum / len(seq)^2 ... let's follow Python exactly:
        // maxes has len(seq)*len(sequences) entries; for 2 seqs = len(seq)*1 = len(seq)
        // result = sum(maxes) / len(seq) / len(maxes)
        //        = sum(maxes) / len(seq) / len(seq)   [since len(maxes)=len(seq) here]
        let total: f64 = maxes.iter().sum();
        total / seq.len() as f64 / maxes.len() as f64
    }

    /// Compute MongeElkan similarity between two strings.
    pub fn similarity(&self, s1: &str, s2: &str) -> f64 {
        // quick_answer equivalent for BaseSimilarity
        if s1 == s2 {
            return 1.0;
        }
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }

        let tokens1 = Self::tokenise(s1, self.qval);
        let tokens2 = Self::tokenise(s2, self.qval);

        if self.symmetric {
            // average over both permutations: (calc(t1,t2) + calc(t2,t1)) / 2
            let r1 = Self::calc(&tokens1, &tokens2);
            let r2 = Self::calc(&tokens2, &tokens1);
            (r1 + r2) / 2.0
        } else {
            Self::calc(&tokens1, &tokens2)
        }
    }

    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.similarity(s1, s2)
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        self.similarity(s1, s2)
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }
}

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

    // ── MongeElkan ───────────────────────────────────────────────────────────

    #[test]
    fn test_monge_elkan_identical() {
        let alg = MongeElkan::default();
        assert_eq!(alg.similarity("test", "test"), 1.0);
    }

    #[test]
    fn test_monge_elkan_empty() {
        let alg = MongeElkan::default();
        assert_eq!(alg.similarity("test", ""), 0.0);
        assert_eq!(alg.similarity("", "test"), 0.0);
    }

    #[test]
    fn test_monge_elkan_normalized() {
        let alg = MongeElkan::default();
        let sim = alg.similarity("test", "text");
        let dist = alg.distance("test", "text");
        assert!((sim + dist - 1.0).abs() < 1e-9);
        assert!((0.0..=1.0).contains(&sim));
    }

    #[test]
    fn test_monge_elkan_basic_similarity() {
        // With DamerauLevenshtein as inner algorithm:
        // comparing char-by-char tokens, closer strings score higher
        let alg = MongeElkan::default();
        let close = alg.similarity("test", "text");
        let far = alg.similarity("test", "xxxx");
        assert!(close > far, "close={close} should be > far={far}");
    }

    #[test]
    fn test_monge_elkan_symmetric() {
        // symmetric=true should average both directions
        let alg_sym = MongeElkan::new(true, 1);
        let alg_asym = MongeElkan::default();
        let s1 = "test";
        let s2 = "testing";
        let sym = alg_sym.similarity(s1, s2);
        let asym = alg_asym.similarity(s1, s2);
        // symmetric averages both directions, so they differ for unequal-length inputs
        // both should be in [0,1]
        assert!((0.0..=1.0).contains(&sym), "sym={sym}");
        assert!((0.0..=1.0).contains(&asym), "asym={asym}");
    }
}
