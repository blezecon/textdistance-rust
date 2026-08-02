//! Sequence-based distance algorithms (Person 1 assignment).
//!
//! Ports the Python `textdistance.algorithms.sequence_based` module:
//! `LCSSeq`, `LCSStr` and `RatcliffObershelp`.

use std::collections::HashMap;

use crate::base::{Base, is_ident};
use crate::types::TestFunc;
use crate::utils::find_ngrams;

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Longest common substring via a difflib-style scan.
///
/// Returns `(start_a, start_b, size)` of the longest common block. Mirrors
/// Python's `difflib.SequenceMatcher.find_longest_match` with no junk and the
/// default `autojunk` heuristic: among maximal blocks the earliest start in
/// `s1` (then `s2`) wins, and the block is then extended both ways.
fn find_longest_match<U: PartialEq>(s1: &[U], s2: &[U]) -> (usize, usize, usize) {
    let mut j2len: HashMap<usize, usize> = HashMap::new();
    let (mut besti, mut bestj, mut bestsize) = (0, 0, 0);
    for (i, c1) in s1.iter().enumerate() {
        let mut new_j2len: HashMap<usize, usize> = HashMap::new();
        for (j, c2) in s2.iter().enumerate() {
            if c1 != c2 {
                continue;
            }
            let prev = j
                .checked_sub(1)
                .and_then(|p| j2len.get(&p))
                .copied()
                .unwrap_or(0);
            let k = prev + 1;
            new_j2len.insert(j, k);
            if k > bestsize {
                besti = i + 1 - k;
                bestj = j + 1 - k;
                bestsize = k;
            }
        }
        j2len = new_j2len;
    }
    while besti > 0 && bestj > 0 && s1[besti - 1] == s2[bestj - 1] {
        besti -= 1;
        bestj -= 1;
        bestsize += 1;
    }
    while besti + bestsize < s1.len()
        && bestj + bestsize < s2.len()
        && s1[besti + bestsize] == s2[bestj + bestsize]
    {
        bestsize += 1;
    }
    (besti, bestj, bestsize)
}

/// Whether `needle` occurs as a contiguous block of `hay`.
fn contains_subslice<U: PartialEq>(hay: &[U], needle: &[U]) -> bool {
    needle.is_empty() || hay.windows(needle.len()).any(|w| w == needle)
}

/// Index of the first occurrence of `needle` in `hay` (mirrors `str.find`).
fn first_index_of<U: PartialEq>(hay: &[U], needle: &[U]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > hay.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Longest common substring by scanning n-grams of the shortest sequence.
///
/// Mirrors the Python `LCSStr._custom`: from longest to shortest, return the
/// first n-gram of the shortest sequence present in all sequences.
fn lcs_str_custom<U: Clone + PartialEq>(seqs: &[&[U]]) -> Vec<U> {
    let Some(&short) = seqs.iter().min_by_key(|s| s.len()) else {
        return Vec::new();
    };
    for n in (1..=short.len()).rev() {
        for sub in short.windows(n) {
            if seqs.iter().all(|s| contains_subslice(s, sub)) {
                return sub.to_vec();
            }
        }
    }
    Vec::new()
}

/// Longest common substring of two sequences, dispatching like Python's
/// `LCSStr.__call__`: a difflib scan for short inputs, the n-gram scan
/// otherwise (the difflib `autojunk` heuristic never applies because it only
/// activates for sequences of 200+ elements, which take the n-gram path).
fn lcs_str_between<U: Clone + PartialEq>(s1: &[U], s2: &[U]) -> Vec<U> {
    if s1.len().max(s2.len()) < 200 {
        let (i, _, k) = find_longest_match(s1, s2);
        s1[i..i + k].to_vec()
    } else {
        lcs_str_custom(&[s1, s2])
    }
}

/// Longest common subsequence length via classic DP over element equality.
fn lcs_seq_len<U: PartialEq>(s1: &[U], s2: &[U]) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut lengths = vec![vec![0usize; len2 + 1]; len1 + 1];
    for i in 1..=len1 {
        for j in 1..=len2 {
            if s1[i - 1] == s2[j - 1] {
                lengths[i][j] = lengths[i - 1][j - 1] + 1;
            } else {
                lengths[i][j] = lengths[i - 1][j].max(lengths[i][j - 1]);
            }
        }
    }
    lengths[len1][len2]
}

/// Multi-sequence LCS length via recursion, mirroring Python `LCSSeq._recursive`.
///
/// When all last elements match, one element is consumed; otherwise each
/// sequence is tried with its last element dropped and the best kept. Two
/// sequences bottom out in the equality-based DP, exactly as in Python.
///
/// Deviation: Python passes every last element to the variadic `test_func`;
/// here each element is tested pairwise against the first sequence's last
/// element (equivalent for the default identity test).
fn lcs_seq_len_recursive<U: Clone + PartialEq>(
    seqs: &[Vec<U>],
    test: impl Fn(&U, &U) -> bool + Copy,
) -> usize {
    if seqs.iter().any(|s| s.is_empty()) {
        return 0;
    }
    if seqs.len() == 2 {
        return lcs_seq_len(&seqs[0], &seqs[1]);
    }
    let last0 = &seqs[0][seqs[0].len() - 1];
    if seqs.iter().all(|s| test(&s[s.len() - 1], last0)) {
        let shortened: Vec<Vec<U>> = seqs.iter().map(|s| s[..s.len() - 1].to_vec()).collect();
        return 1 + lcs_seq_len_recursive(&shortened, test);
    }
    let mut best = 0;
    for i in 0..seqs.len() {
        let mut ss = seqs.to_vec();
        ss[i].pop();
        best = best.max(lcs_seq_len_recursive(&ss, test));
    }
    best
}

// ─── LCSSeq ──────────────────────────────────────────────────────────────────

/// Longest common subsequence similarity.
///
/// The similarity is the length of the longest subsequence common to both
/// inputs. Two inputs are handled by the DP; `similarity_multi` extends this
/// to any number of inputs via recursion.
///
/// * `qval`      – q-gram tokenization (1 = per element, the default).
/// * `test_func` – used only by the multi-input recursion; the two-input DP
///   compares elements by equality, mirroring the Python reference.
#[derive(Debug, Clone, Copy)]
pub struct LCSSeq<T> {
    pub qval: usize,
    pub test_func: TestFunc<T>,
}

impl<T> Default for LCSSeq<T> {
    fn default() -> Self {
        Self {
            qval: 1,
            test_func: None,
        }
    }
}

impl<T> LCSSeq<T> {
    /// Create a `LCSSeq` metric.
    pub fn new(qval: usize, test_func: TestFunc<T>) -> Self {
        Self { qval, test_func }
    }
}

impl<T: Clone + PartialEq> LCSSeq<T> {
    /// LCS length over any number of sequences.
    pub fn similarity_multi(&self, seqs: &[Vec<T>]) -> usize {
        match seqs.len() {
            0 => 0,
            1 => seqs[0].len(),
            2 => lcs_seq_len(&seqs[0], &seqs[1]),
            _ => {
                let test = self.test_func.unwrap_or(|a, b| a == b);
                lcs_seq_len_recursive(seqs, test)
            }
        }
    }
}

impl<T: Clone + PartialEq> Base<T> for LCSSeq<T> {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            lcs_seq_len(s1, s2) as f64
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            lcs_seq_len(&g1, &g2) as f64
        }
    }
}

// ─── LCSStr ──────────────────────────────────────────────────────────────────

/// Longest common substring similarity.
///
/// The similarity is the length of the longest substring common to both
/// inputs. Two short inputs use a difflib-style scan, while long and
/// multi-input cases scan n-grams of the shortest input (mirroring Python's
/// dispatch in `LCSStr.__call__`).
///
/// * `qval` – q-gram tokenization (1 = per element, the default).
#[derive(Debug, Clone, Copy, Default)]
pub struct LCSStr {
    pub qval: usize,
}

impl LCSStr {
    /// Create a `LCSStr` metric.
    pub fn new(qval: usize) -> Self {
        Self { qval }
    }

    /// LCS substring length over any number of sequences.
    pub fn similarity_multi<T: Clone + PartialEq>(&self, seqs: &[Vec<T>]) -> usize {
        if seqs.is_empty() || seqs.iter().any(|s| s.is_empty()) {
            return 0;
        }
        if seqs.len() == 1 {
            return seqs[0].len();
        }
        if seqs.len() == 2 && seqs[0].len().max(seqs[1].len()) < 200 {
            return find_longest_match(&seqs[0], &seqs[1]).2;
        }
        let refs: Vec<&[T]> = seqs.iter().map(|s| s.as_slice()).collect();
        lcs_str_custom(&refs).len()
    }
}

impl<T: Clone + PartialEq> Base<T> for LCSStr {
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            lcs_str_between(s1, s2).len() as f64
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            lcs_str_between(&g1, &g2).len() as f64
        }
    }
}

// ─── RatcliffObershelp ───────────────────────────────────────────────────────

/// Recursive sum of longest-common-substring lengths.
///
/// Mirrors the Python `RatcliffObershelp._find`: find the longest common
/// substring, sum its length with the recursed sums of the parts left and
/// right of its first occurrence in each sequence.
fn ratcliff_find<U: Clone + PartialEq>(s1: &[U], s2: &[U]) -> usize {
    let subseq = lcs_str_between(s1, s2);
    let length = subseq.len();
    if length == 0 {
        return 0;
    }
    let pos1 = first_index_of(s1, &subseq).unwrap_or(0);
    let pos2 = first_index_of(s2, &subseq).unwrap_or(0);
    ratcliff_find(&s1[..pos1], &s2[..pos2])
        + length
        + ratcliff_find(&s1[pos1 + length..], &s2[pos2 + length..])
}

/// Ratcliff-Obershelp (gestalt pattern matching) similarity.
///
/// The similarity is twice the recursive longest-common-substring total
/// divided by the summed input lengths. Identical inputs score 1 and empty
/// inputs score 0, mirroring the Python `quick_answer`.
///
/// * `qval` – q-gram tokenization (1 = per element, the default).
#[derive(Debug, Clone, Copy, Default)]
pub struct RatcliffObershelp {
    pub qval: usize,
}

impl RatcliffObershelp {
    /// Create a `RatcliffObershelp` metric.
    pub fn new(qval: usize) -> Self {
        Self { qval }
    }
}

impl<T: Clone + PartialEq> Base<T> for RatcliffObershelp {
    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if is_ident(s1, s2) {
            return 1.0;
        }
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }
        let found = if self.qval <= 1 {
            ratcliff_find(s1, s2)
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            ratcliff_find(&g1, &g2)
        };
        2.0 * found as f64 / (s1.len() + s2.len()) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    fn assert_close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "expected {a}, got {b}");
    }

    #[test]
    fn test_lcsseq_python_values() {
        let l = LCSSeq::<char>::default();
        let cases = [
            ("ab", "cd", 0.0),
            ("abcd", "abcd", 4.0),
            ("test", "text", 3.0),
            ("thisisatest", "testing123testing", 7.0),
            ("DIXON", "DICKSONX", 4.0),
            ("random exponential", "layer activation", 5.0),
            ("a", "aa", 1.0),
            ("a", "b", 0.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(l.similarity(&chars(a), &chars(b)), expected);
        }
        let long = "a".repeat(80);
        assert_eq!(l.similarity(&chars(&long), &chars(&long)), 80.0);
        let long_a = "a".repeat(80);
        let long_b = "b".repeat(80);
        assert_eq!(l.similarity(&chars(&long_a), &chars(&long_b)), 0.0);
    }

    #[test]
    fn test_lcsseq_edge_cases() {
        let l = LCSSeq::<char>::default();
        assert_eq!(l.similarity(&chars(""), &chars("")), 0.0);
        assert_eq!(l.similarity(&chars(""), &chars("abc")), 0.0);
        assert_eq!(l.distance(&chars("abc"), &chars("abc")), 0.0);
        assert_eq!(l.distance(&chars("ab"), &chars("cd")), 2.0);
    }

    #[test]
    fn test_lcsseq_multi() {
        let l = LCSSeq::<char>::default();
        let cases: Vec<(Vec<&str>, usize)> = vec![
            (vec!["a", "b", "c"], 0),
            (vec!["a", "a", "a"], 1),
            (vec!["test", "text", "tempest"], 3),
        ];
        for (seqs, expected) in cases {
            let owned: Vec<Vec<char>> = seqs.iter().map(|s| chars(s)).collect();
            assert_eq!(l.similarity_multi(&owned), expected);
        }
    }

    #[test]
    fn test_lcsseq_qval() {
        // Python crashes with qval > 1 (`TypeError: can only concatenate
        // tuple to tuple`); here the LCS length over q-gram tokens is
        // computed instead.
        let l = LCSSeq::new(2, None);
        assert_eq!(l.similarity(&chars("hello"), &chars("hello")), 4.0);
        assert_eq!(l.similarity(&chars("hello"), &chars("helxo")), 2.0);
    }

    #[test]
    fn test_lcsstr_qval() {
        let l = LCSStr::new(2);
        assert_eq!(l.similarity(&chars("hello"), &chars("hello")), 4.0);
        assert_eq!(l.similarity(&chars("hello"), &chars("helxo")), 2.0);
        assert_eq!(l.similarity(&chars("test"), &chars("text")), 1.0);
    }

    #[test]
    fn test_lcsstr_python_values() {
        let l = LCSStr::default();
        let cases = [
            ("ab", "abcd", 2.0),
            ("abcd", "ab", 2.0),
            ("abcd", "bc", 2.0),
            ("bc", "abcd", 2.0),
            ("abcd", "cd", 2.0),
            ("abcd", "ef", 0.0),
            ("ef", "abcd", 0.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(l.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_lcsstr_long_inputs() {
        let l = LCSStr::default();
        let a = chars("MYTEST").repeat(100);
        let b = chars("TEST");
        assert_eq!(l.similarity(&a, &b), 4.0);
        assert_eq!(l.similarity(&b, &a), 4.0);
    }

    #[test]
    fn test_lcsstr_edge_cases() {
        let l = LCSStr::default();
        assert_eq!(l.similarity(&chars(""), &chars("")), 0.0);
        assert_eq!(l.similarity(&chars(""), &chars("abc")), 0.0);
        assert_eq!(l.distance(&chars("abcd"), &chars("ef")), 4.0);
        let abc = chars("abc");
        assert_eq!(l.similarity_multi(&[abc.clone(), abc]), 3);
        assert_eq!(l.similarity_multi(&[chars("abcd")]), 4);
    }

    #[test]
    fn test_lcsstr_multi() {
        let l = LCSStr::default();
        let cases: Vec<(Vec<&str>, usize)> = vec![
            (vec!["ab", "abcd", "abxy"], 2),
            (vec!["abc", "def", "ghi"], 0),
            (vec!["test", "text", "tempest"], 2),
        ];
        for (seqs, expected) in cases {
            let owned: Vec<Vec<char>> = seqs.iter().map(|s| chars(s)).collect();
            assert_eq!(l.similarity_multi(&owned), expected);
        }
    }

    #[test]
    fn test_ratcliff_obershelp_python_values() {
        let r = RatcliffObershelp::default();
        let cases = [
            ("ab", "abcd", 0.6666666666666666),
            ("test", "text", 0.75),
            ("MARTHA", "MARHTA", 0.8333333333333334),
            ("", "abc", 0.0),
            ("abc", "abc", 1.0),
            ("hello", "haloa", 0.6),
            ("DIXON", "DICKSONX", 0.6153846153846154),
            ("DWAYNE", "DUANE", 0.7272727272727273),
        ];
        for (a, b, expected) in cases {
            assert_close(r.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_ratcliff_obershelp_long_address() {
        let r = RatcliffObershelp::default();
        assert_close(
            r.similarity(
                &chars("sint-pietersplein 6, 9000 gent"),
                &chars("test 10, 1010 brussel"),
            ),
            0.39215686274509803,
        );
    }

    #[test]
    fn test_ratcliff_obershelp_edge_cases() {
        let r = RatcliffObershelp::default();
        assert_eq!(r.distance(&chars(""), &chars("")), 0.0);
        assert_eq!(r.similarity(&chars(""), &chars("")), 1.0);
        assert_eq!(r.similarity(&chars(""), &chars("abc")), 0.0);
        assert_eq!(r.similarity(&chars("spam"), &chars("qwer")), 0.0);
        assert_eq!(r.distance(&chars(""), &chars("qwertyui")), 1.0);
    }

    #[test]
    fn test_sequence_normalization_invariants() {
        let algs: Vec<Box<dyn Base<char>>> = vec![
            Box::new(LCSSeq::<char>::default()),
            Box::new(LCSStr::default()),
            Box::new(RatcliffObershelp::default()),
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
