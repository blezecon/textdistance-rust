//! Edit-based distance metrics (Person 1 assignment).
//!
//! Ports the Python `textdistance.algorithms.edit_based` module. Part 2 covers
//! [`Hamming`], [`Levenshtein`] and [`DamerauLevenshtein`].

use std::collections::HashMap;
use std::hash::Hash;

use crate::base::{Base, is_ident};
use crate::types::{SimFunc, TestFunc};
use crate::utils::find_ngrams;

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Count mismatching positions between two token sequences.
///
/// When `truncate` is set only the common prefix is compared; otherwise the
/// length difference is counted as mismatches (mirroring Python's
/// `zip_longest` with a default identity test).
fn count_mismatches<U>(s1: &[U], s2: &[U], truncate: bool, test: impl Fn(&U, &U) -> bool) -> f64 {
    let common = s1
        .iter()
        .zip(s2.iter())
        .filter(|(a, b)| !test(a, b))
        .count();
    let diff = s1.len().abs_diff(s2.len());
    if truncate {
        common as f64
    } else {
        (common + diff) as f64
    }
}

/// Classic two-row Levenshtein DP over token sequences.
fn levenshtein_dp<U>(s1: &[U], s2: &[U], test: impl Fn(&U, &U) -> bool) -> f64 {
    let cols = s2.len() + 1;
    let mut prev: Vec<usize> = (0..cols).collect();
    let mut cur: Vec<usize> = vec![0; cols];
    for (r, ch1) in s1.iter().enumerate() {
        cur[0] = r + 1;
        for (c, ch2) in s2.iter().enumerate() {
            let deletion = prev[c + 1] + 1;
            let insertion = cur[c] + 1;
            let substitution = prev[c] + usize::from(!test(ch1, ch2));
            cur[c + 1] = substitution.min(deletion).min(insertion);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[cols - 1] as f64
}

/// Restricted (OSA) Damerau-Levenshtein DP (transposition of adjacent chars).
fn damerau_restricted<U>(s1: &[U], s2: &[U], test: impl Fn(&U, &U) -> bool) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut d = vec![vec![0usize; len2 + 1]; len1 + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = usize::from(!test(&s1[i - 1], &s2[j - 1]));
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && test(&s1[i - 1], &s2[j - 2]) && test(&s1[i - 2], &s2[j - 1]) {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + cost);
            }
        }
    }
    d[len1][len2]
}

/// Unrestricted Damerau-Levenshtein DP (Wikipedia algorithm with `da` map).
fn damerau_unrestricted<U: Hash + Eq + Clone>(
    s1: &[U],
    s2: &[U],
    test: impl Fn(&U, &U) -> bool,
) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let maxdist = len1 + len2;
    let mut d = vec![vec![0usize; len2 + 1]; len1 + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    let mut da: HashMap<U, usize> = HashMap::new();
    for i in 1..=len1 {
        let mut db = 0;
        let cs1 = &s1[i - 1];
        for j in 1..=len2 {
            let cs2 = &s2[j - 1];
            let i1 = da.get(cs2).copied().unwrap_or(0);
            let j1 = db;
            let cost;
            if test(cs1, cs2) {
                cost = 0;
                db = j;
            } else {
                cost = 1;
            }
            let transposition = {
                let base = if i1 == 0 || j1 == 0 {
                    maxdist
                } else {
                    d[i1 - 1][j1 - 1]
                };
                base + (i - i1) - 1 + (j - j1)
            };
            d[i][j] = (d[i - 1][j - 1] + cost)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j] + 1)
                .min(transposition);
        }
        da.insert(cs1.clone(), i);
    }
    d[len1][len2]
}

// ─── Hamming ─────────────────────────────────────────────────────────────────

/// Hamming distance: the number of differing positions in ordered sequences.
///
/// * `qval`     – q-gram tokenization (1 = per element, the default).
/// * `test_func`– custom element test; defaults to equality.
/// * `truncate` – compare only the common prefix (`zip` instead of
///   `zip_longest`); otherwise the length difference counts as mismatches.
///
/// The `external` flag from Python is not ported.
#[derive(Debug, Clone, Copy)]
pub struct Hamming<T> {
    pub qval: usize,
    pub test_func: TestFunc<T>,
    pub truncate: bool,
}

impl<T> Default for Hamming<T> {
    fn default() -> Self {
        Self {
            qval: 1,
            test_func: None,
            truncate: false,
        }
    }
}

impl<T> Hamming<T> {
    /// Create a `Hamming` metric.
    pub fn new(qval: usize, test_func: TestFunc<T>, truncate: bool) -> Self {
        Self {
            qval,
            test_func,
            truncate,
        }
    }
}

impl<T: Clone + PartialEq> Base<T> for Hamming<T> {
    fn distance(&self, s1: &[T], s2: &[T]) -> f64 {
        if is_ident(s1, s2) {
            return 0.0;
        }
        if self.qval <= 1 {
            if s1.is_empty() || s2.is_empty() {
                return s1.len().max(s2.len()) as f64;
            }
            let test = self.test_func.unwrap_or(|a, b| a == b);
            count_mismatches(s1, s2, self.truncate, test)
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            if g1.is_empty() || g2.is_empty() {
                return g1.len().max(g2.len()) as f64;
            }
            count_mismatches(&g1, &g2, self.truncate, |a, b| a == b)
        }
    }
}

// ─── Levenshtein ─────────────────────────────────────────────────────────────

/// Levenshtein edit distance: the minimum number of deletions, insertions and
/// substitutions to transform one sequence into the other.
///
/// * `qval`      – q-gram tokenization (1 = per element, the default).
/// * `test_func` – custom element test; defaults to equality.
///
/// For `qval > 1` the distance is computed over q-gram tokens compared by
/// equality (the Python `test_func` operates on whole q-gram tuples).
#[derive(Debug, Clone, Copy)]
pub struct Levenshtein<T> {
    pub qval: usize,
    pub test_func: TestFunc<T>,
}

impl<T> Default for Levenshtein<T> {
    fn default() -> Self {
        Self {
            qval: 1,
            test_func: None,
        }
    }
}

impl<T> Levenshtein<T> {
    /// Create a `Levenshtein` metric.
    pub fn new(qval: usize, test_func: TestFunc<T>) -> Self {
        Self { qval, test_func }
    }
}

impl<T: Clone + PartialEq> Base<T> for Levenshtein<T> {
    fn distance(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            let test = self.test_func.unwrap_or(|a, b| a == b);
            levenshtein_dp(s1, s2, test)
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            levenshtein_dp(&g1, &g2, |a, b| a == b)
        }
    }
}

// ─── DamerauLevenshtein ──────────────────────────────────────────────────────

/// Damerau-Levenshtein edit distance: Levenshtein plus adjacent transpositions.
///
/// * `restricted` – `true` uses the restricted (OSA) variant, `false` the
///   unrestricted one where the same character may be touched more than once.
/// * `qval`       – q-gram tokenization (1 = per element, the default).
/// * `test_func`  – custom element test; defaults to equality.
///
/// Person 2's `MongeElkan` imports this type.
#[derive(Debug, Clone, Copy)]
pub struct DamerauLevenshtein<T> {
    pub qval: usize,
    pub test_func: TestFunc<T>,
    pub restricted: bool,
}

impl<T> Default for DamerauLevenshtein<T> {
    fn default() -> Self {
        Self {
            qval: 1,
            test_func: None,
            restricted: true,
        }
    }
}

impl<T> DamerauLevenshtein<T> {
    /// Create a `DamerauLevenshtein` metric.
    pub fn new(qval: usize, test_func: TestFunc<T>, restricted: bool) -> Self {
        Self {
            qval,
            test_func,
            restricted,
        }
    }
}

impl<T: Clone + Hash + Eq> Base<T> for DamerauLevenshtein<T> {
    fn distance(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            let test = self.test_func.unwrap_or(|a, b| a == b);
            if self.restricted {
                damerau_restricted(s1, s2, test) as f64
            } else {
                damerau_unrestricted(s1, s2, test) as f64
            }
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            let test = |a: &Vec<T>, b: &Vec<T>| a == b;
            if self.restricted {
                damerau_restricted(&g1, &g2, test) as f64
            } else {
                damerau_unrestricted(&g1, &g2, test) as f64
            }
        }
    }
}

// ─── Jaro / JaroWinkler ──────────────────────────────────────────────────────

/// Core Jaro similarity computation over token sequences.
///
/// Mirrors the Python `JaroWinkler.__call__` with `winklerize` and
/// `long_tolerance` options and a configurable `prefix_weight`.
fn jaro_core<U: PartialEq>(
    s1: &[U],
    s2: &[U],
    long_tolerance: bool,
    winklerize: bool,
    prefix_weight: f64,
) -> f64 {
    if is_ident(s1, s2) {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let s1_len = s1.len();
    let s2_len = s2.len();
    let min_len = s1_len.min(s2_len);
    let search_range = (s1_len.max(s2_len) / 2).saturating_sub(1);

    let mut s1_flags = vec![false; s1_len];
    let mut s2_flags = vec![false; s2_len];

    let mut common_chars = 0;
    for (i, s1_ch) in s1.iter().enumerate() {
        let low = i.saturating_sub(search_range);
        let hi = (i + search_range).min(s2_len - 1);
        for j in low..=hi {
            if !s2_flags[j] && s2[j] == *s1_ch {
                s1_flags[i] = true;
                s2_flags[j] = true;
                common_chars += 1;
                break;
            }
        }
    }

    if common_chars == 0 {
        return 0.0;
    }

    let mut k = 0usize;
    let mut trans_count = 0usize;
    for (i, &s1_f) in s1_flags.iter().enumerate() {
        if !s1_f {
            continue;
        }
        let mut j = 0usize;
        if let Some(jj) = (k..s2_len).find(|&jj| s2_flags[jj]) {
            k = jj + 1;
            j = jj;
        }
        if s1[i] != s2[j] {
            trans_count += 1;
        }
    }
    trans_count /= 2;

    let common = common_chars as f64;
    let mut weight = common / s1_len as f64 + common / s2_len as f64;
    weight += (common - trans_count as f64) / common;
    weight /= 3.0;

    if !winklerize {
        return weight;
    }
    if weight <= 0.7 {
        return weight;
    }

    let j = min_len.min(4);
    let mut i = 0;
    while i < j && s1[i] == s2[i] {
        i += 1;
    }
    if i > 0 {
        weight += i as f64 * prefix_weight * (1.0 - weight);
    }

    if !long_tolerance || min_len <= 4 {
        return weight;
    }
    if common_chars <= i + 1 || 2 * common_chars < min_len + i {
        return weight;
    }
    let tmp = (common_chars - i - 1) as f64 / (s1_len + s2_len - i * 2 + 2) as f64;
    weight += (1.0 - weight) * tmp;
    weight
}

/// Jaro-Winkler similarity measure.
///
/// Jaro similarity with an optional boost for common prefixes
/// (`winklerize`) and an optional adjustment for long strings
/// (`long_tolerance`).
///
/// Person 2's `MongeElkan` imports this type for its test cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JaroWinkler {
    pub long_tolerance: bool,
    pub winklerize: bool,
    pub qval: usize,
}

impl Default for JaroWinkler {
    fn default() -> Self {
        Self {
            long_tolerance: false,
            winklerize: true,
            qval: 1,
        }
    }
}

impl JaroWinkler {
    /// Create a `JaroWinkler` metric.
    pub fn new(long_tolerance: bool, winklerize: bool, qval: usize) -> Self {
        Self {
            long_tolerance,
            winklerize,
            qval,
        }
    }
}

impl<T: Clone + PartialEq> Base<T> for JaroWinkler {
    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            jaro_core(s1, s2, self.long_tolerance, self.winklerize, 0.1)
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            jaro_core(&g1, &g2, self.long_tolerance, self.winklerize, 0.1)
        }
    }
}

/// Jaro similarity measure: [`JaroWinkler`] with `winklerize = false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jaro {
    pub long_tolerance: bool,
    pub qval: usize,
}

impl Default for Jaro {
    fn default() -> Self {
        Self {
            long_tolerance: false,
            qval: 1,
        }
    }
}

impl Jaro {
    /// Create a `Jaro` metric.
    pub fn new(long_tolerance: bool, qval: usize) -> Self {
        Self {
            long_tolerance,
            qval,
        }
    }
}

impl<T: Clone + PartialEq> Base<T> for Jaro {
    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            jaro_core(s1, s2, self.long_tolerance, false, 0.1)
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            jaro_core(&g1, &g2, self.long_tolerance, false, 0.1)
        }
    }
}

// ─── StrCmp95 ────────────────────────────────────────────────────────────────

/// Pairs of visually or phonetically similar characters given partial credit.
const SP_MX: [(char, char); 36] = [
    ('A', 'E'),
    ('A', 'I'),
    ('A', 'O'),
    ('A', 'U'),
    ('B', 'V'),
    ('E', 'I'),
    ('E', 'O'),
    ('E', 'U'),
    ('I', 'O'),
    ('I', 'U'),
    ('O', 'U'),
    ('I', 'Y'),
    ('E', 'Y'),
    ('C', 'G'),
    ('E', 'F'),
    ('W', 'U'),
    ('W', 'V'),
    ('X', 'K'),
    ('S', 'Z'),
    ('X', 'S'),
    ('Q', 'C'),
    ('U', 'V'),
    ('M', 'N'),
    ('L', 'I'),
    ('Q', 'O'),
    ('P', 'R'),
    ('I', 'J'),
    ('2', 'Z'),
    ('5', 'S'),
    ('8', 'B'),
    ('1', 'I'),
    ('1', 'L'),
    ('0', 'O'),
    ('0', 'Q'),
    ('C', 'K'),
    ('G', 'J'),
];

/// True when the character's code point is in `(0, 91)`, matching Python.
fn strcmp_in_range(c: char) -> bool {
    let ord = c as u32;
    ord > 0 && ord < 91
}

/// Strip whitespace and uppercase, mirroring Python's `str.strip().upper()`.
fn strcmp_preprocess(s: &[char]) -> Vec<char> {
    let start = s.iter().position(|c| !c.is_whitespace()).unwrap_or(s.len());
    let end = s
        .iter()
        .rposition(|c| !c.is_whitespace())
        .map_or(0, |i| i + 1);
    let trimmed = if start < end { &s[start..end] } else { &[] };
    trimmed
        .iter()
        .map(|c| c.to_uppercase().next().unwrap_or(*c))
        .collect()
}

/// strcmp95 similarity, a Jaro-Winkler variant with phonetic/OCR partial
/// credit. Operates on uppercase ASCII `char` sequences.
///
/// * `long_strings` – enable the optional long-string boost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StrCmp95 {
    pub long_strings: bool,
}

impl StrCmp95 {
    /// Create a `StrCmp95` metric.
    pub fn new(long_strings: bool) -> Self {
        Self { long_strings }
    }
}

impl Base<char> for StrCmp95 {
    fn maximum(&self, _s1: &[char], _s2: &[char]) -> f64 {
        1.0
    }

    fn similarity(&self, s1: &[char], s2: &[char]) -> f64 {
        let s1 = strcmp_preprocess(s1);
        let s2 = strcmp_preprocess(s2);

        if is_ident(&s1, &s2) {
            return 1.0;
        }
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }

        let len_s1 = s1.len();
        let len_s2 = s2.len();

        let mut adjwt = HashMap::new();
        for (c1, c2) in SP_MX {
            adjwt.insert((c1, c2), 3);
            adjwt.insert((c2, c1), 3);
        }

        let (search_range0, minv) = if len_s1 > len_s2 {
            (len_s1, len_s2)
        } else {
            (len_s2, len_s1)
        };

        let mut s1_flag = vec![0i32; search_range0];
        let mut s2_flag = vec![0i32; search_range0];
        let search_range = (search_range0 / 2).saturating_sub(1);

        let yl1 = len_s2 - 1;
        let mut num_com = 0usize;
        for (i, &sc1) in s1.iter().enumerate() {
            let lowlim = i.saturating_sub(search_range);
            let hilim = (i + search_range).min(yl1);
            for j in lowlim..=hilim {
                if s2_flag[j] == 0 && s2[j] == sc1 {
                    s2_flag[j] = 1;
                    s1_flag[i] = 1;
                    num_com += 1;
                    break;
                }
            }
        }

        if num_com == 0 {
            return 0.0;
        }

        let mut k = 0usize;
        let mut n_trans = 0usize;
        for (i, &sc1) in s1.iter().enumerate() {
            if s1_flag[i] == 0 {
                continue;
            }
            let mut j = 0usize;
            if let Some(jj) = (k..len_s2).find(|&jj| s2_flag[jj] != 0) {
                k = jj + 1;
                j = jj;
            }
            if sc1 != s2[j] {
                n_trans += 1;
            }
        }
        n_trans /= 2;

        let mut n_simi = 0i32;
        if minv > num_com {
            for i in 0..len_s1 {
                if s1_flag[i] != 0 || !strcmp_in_range(s1[i]) {
                    continue;
                }
                for j in 0..len_s2 {
                    if s2_flag[j] != 0 || !strcmp_in_range(s2[j]) {
                        continue;
                    }
                    if !adjwt.contains_key(&(s1[i], s2[j])) {
                        continue;
                    }
                    n_simi += adjwt[&(s1[i], s2[j])];
                    s2_flag[j] = 2;
                    break;
                }
            }
        }
        let num_sim = n_simi as f64 / 10.0 + num_com as f64;

        let mut weight = num_sim / len_s1 as f64 + num_sim / len_s2 as f64;
        weight += (num_com as f64 - n_trans as f64) / num_com as f64;
        weight /= 3.0;

        if weight <= 0.7 {
            return weight;
        }

        let j = minv.min(4);
        let mut i = 0usize;
        for (sc1, sc2) in s1.iter().zip(s2.iter()) {
            if i >= j {
                break;
            }
            if sc1 != sc2 {
                break;
            }
            if sc1.is_ascii_digit() {
                break;
            }
            i += 1;
        }
        if i > 0 {
            weight += i as f64 * 0.1 * (1.0 - weight);
        }

        if !self.long_strings {
            return weight;
        }
        if minv <= 4 {
            return weight;
        }
        if num_com <= i + 1 || 2 * num_com < minv + i {
            return weight;
        }
        if s1[0].is_ascii_digit() {
            return weight;
        }
        let res = (num_com - i - 1) as f64 / (len_s1 + len_s2 - i * 2 + 2) as f64;
        weight += (1.0 - weight) * res;
        weight
    }
}

// ─── MLIPNS ──────────────────────────────────────────────────────────────────

/// Core MLIPNS loop over a Hamming distance and maximum length.
fn mlipns_core(ham: f64, maxlen: usize, threshold: f64, maxmismatches: usize) -> f64 {
    let mut ham = ham as i64;
    let mut maxlen = maxlen as i64;
    let mut mismatches: i64 = 0;
    while mismatches <= maxmismatches as i64 {
        if maxlen == 0 {
            return 1.0;
        }
        if 1.0 - (maxlen as f64 - ham as f64) / maxlen as f64 <= threshold {
            return 1.0;
        }
        mismatches += 1;
        ham -= 1;
        maxlen -= 1;
    }
    if maxlen == 0 { 1.0 } else { 0.0 }
}

/// MLIPNS similarity: iteratively relaxes a Hamming distance until the
/// similarity crosses `threshold` or `maxmismatches` are spent.
///
/// The maximum is always `1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MLIPNS {
    pub threshold: f64,
    pub maxmismatches: usize,
    pub qval: usize,
}

impl Default for MLIPNS {
    fn default() -> Self {
        Self {
            threshold: 0.25,
            maxmismatches: 2,
            qval: 1,
        }
    }
}

impl MLIPNS {
    /// Create an `MLIPNS` metric.
    pub fn new(threshold: f64, maxmismatches: usize, qval: usize) -> Self {
        Self {
            threshold,
            maxmismatches,
            qval,
        }
    }
}

impl<T: Clone + PartialEq> Base<T> for MLIPNS {
    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if is_ident(s1, s2) {
            return 1.0;
        }
        if self.qval <= 1 {
            if s1.is_empty() || s2.is_empty() {
                return 0.0;
            }
            let ham = Hamming::default().distance(s1, s2);
            mlipns_core(
                ham,
                s1.len().max(s2.len()),
                self.threshold,
                self.maxmismatches,
            )
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            if g1.is_empty() || g2.is_empty() {
                return 0.0;
            }
            let ham = count_mismatches(&g1, &g2, false, |a, b| a == b);
            mlipns_core(
                ham,
                g1.len().max(g2.len()),
                self.threshold,
                self.maxmismatches,
            )
        }
    }
}

// ─── NeedlemanWunsch / Gotoh / SmithWaterman ────────────────────────────────

/// Needleman-Wunsch global alignment DP.
///
/// Mirrors the Python numpy implementation with `f64` values: the first row and
/// column are initialized to `-i * gap_cost` and each cell is the max of the
/// diagonal (substitution), up (deletion) and left (insertion) moves.
fn needleman_wunsch_dp<U>(s1: &[U], s2: &[U], gap_cost: f64, sim: impl Fn(&U, &U) -> f64) -> f64 {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut dist = vec![vec![0.0f64; len2 + 1]; len1 + 1];
    for (i, row) in dist.iter_mut().enumerate() {
        row[0] = -(i as f64) * gap_cost;
    }
    for (j, cell) in dist[0].iter_mut().enumerate() {
        *cell = -(j as f64) * gap_cost;
    }
    for i in 1..=len1 {
        for j in 1..=len2 {
            let subst = dist[i - 1][j - 1] + sim(&s1[i - 1], &s2[j - 1]);
            let delete = dist[i - 1][j] - gap_cost;
            let insert = dist[i][j - 1] - gap_cost;
            dist[i][j] = subst.max(delete).max(insert);
        }
    }
    dist[len1][len2]
}

/// Normalized distance for global-alignment metrics, mirroring the Python
/// `(distance - minimum) / (maximum - minimum)` formula with `minimum` and
/// `maximum` supplied by the caller.
fn global_normalized_distance(distance: f64, minimum: f64, maximum: f64) -> f64 {
    if maximum == 0.0 {
        0.0
    } else {
        (distance - minimum) / (maximum - minimum)
    }
}

/// Normalized similarity for global-alignment metrics, mirroring the Python
/// `(similarity - minimum) / (maximum * 2)` formula.
fn global_normalized_similarity(similarity: f64, minimum: f64, maximum: f64) -> f64 {
    if maximum == 0.0 {
        1.0
    } else {
        (similarity - minimum) / (maximum * 2.0)
    }
}

/// Needleman-Wunsch similarity: global alignment score.
///
/// * `gap_cost` – penalty subtracted for each gap.
/// * `sim_func` – element similarity; defaults to 1 for equal, 0 otherwise.
/// * `qval`     – q-gram tokenization (1 = per element, the default).
///
/// Unlike most metrics the raw value is a signed alignment score: `distance`
/// is `-similarity` and the normalized variants rescale between `minimum`
/// (`-max_len * gap_cost`) and `maximum` (`max_len`).
#[derive(Debug, Clone, Copy)]
pub struct NeedlemanWunsch<T> {
    pub qval: usize,
    pub gap_cost: f64,
    pub sim_func: SimFunc<T>,
}

impl<T> Default for NeedlemanWunsch<T> {
    fn default() -> Self {
        Self {
            qval: 1,
            gap_cost: 1.0,
            sim_func: None,
        }
    }
}

impl<T> NeedlemanWunsch<T> {
    /// Create a `NeedlemanWunsch` metric.
    pub fn new(qval: usize, gap_cost: f64, sim_func: SimFunc<T>) -> Self {
        Self {
            qval,
            gap_cost,
            sim_func,
        }
    }

    /// Minimum possible score for the given inputs (`-max_len * gap_cost`).
    pub fn minimum(&self, s1: &[T], s2: &[T]) -> f64 {
        -(s1.len().max(s2.len()) as f64) * self.gap_cost
    }
}

impl<T: Clone + PartialEq> Base<T> for NeedlemanWunsch<T> {
    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            let sim = self
                .sim_func
                .unwrap_or(|a, b| if a == b { 1.0 } else { 0.0 });
            needleman_wunsch_dp(s1, s2, self.gap_cost, sim)
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            needleman_wunsch_dp(
                &g1,
                &g2,
                self.gap_cost,
                |a: &Vec<T>, b: &Vec<T>| {
                    if a == b { 1.0 } else { 0.0 }
                },
            )
        }
    }

    fn distance(&self, s1: &[T], s2: &[T]) -> f64 {
        -self.similarity(s1, s2)
    }

    fn normalized_distance(&self, s1: &[T], s2: &[T]) -> f64 {
        global_normalized_distance(
            self.distance(s1, s2),
            self.minimum(s1, s2),
            self.maximum(s1, s2),
        )
    }

    fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        global_normalized_similarity(
            self.similarity(s1, s2),
            self.minimum(s1, s2),
            self.maximum(s1, s2),
        )
    }
}

/// Gotoh (affine gap) global alignment DP.
///
/// Extends [`NeedlemanWunsch`] with separate gap-open (`gap_open`) and
/// gap-extension (`gap_ext`) penalties tracked in the `p` (gaps in the first
/// sequence) and `q` (gaps in the second sequence) matrices.
///
/// Deviation: the Python reference raises `IndexError` when either sequence is
/// empty; here the pure-gap score `-gap_open - gap_ext * (len - 1)` is
/// returned instead.
fn gotoh_dp<U>(
    s1: &[U],
    s2: &[U],
    gap_open: f64,
    gap_ext: f64,
    sim: impl Fn(&U, &U) -> f64,
) -> f64 {
    let len1 = s1.len();
    let len2 = s2.len();
    if len1 == 0 || len2 == 0 {
        if len1 == 0 && len2 == 0 {
            return 0.0;
        }
        let gaps = if len1 == 0 { len2 } else { len1 };
        return -gap_open - gap_ext * (gaps as f64 - 1.0);
    }

    let neg_inf = f64::NEG_INFINITY;
    let mut d = vec![vec![neg_inf; len2 + 1]; len1 + 1];
    let mut p = vec![vec![neg_inf; len2 + 1]; len1 + 1];
    let mut q = vec![vec![neg_inf; len2 + 1]; len1 + 1];

    d[0][0] = 0.0;
    for i in 1..=len1 {
        p[i][0] = -gap_open - gap_ext * (i as f64 - 1.0);
        q[i][1] = -gap_open;
    }
    for j in 1..=len2 {
        p[1][j] = -gap_open;
        q[0][j] = -gap_open - gap_ext * (j as f64 - 1.0);
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let sim_val = sim(&s1[i - 1], &s2[j - 1]);
            d[i][j] = (d[i - 1][j - 1] + sim_val)
                .max(p[i - 1][j - 1] + sim_val)
                .max(q[i - 1][j - 1] + sim_val);
            p[i][j] = (d[i - 1][j] - gap_open).max(p[i - 1][j] - gap_ext);
            q[i][j] = (d[i][j - 1] - gap_open).max(q[i][j - 1] - gap_ext);
        }
    }
    d[len1][len2].max(p[len1][len2]).max(q[len1][len2])
}

/// Gotoh similarity: Needleman-Wunsch with affine gap penalties.
///
/// * `gap_open` – penalty for opening a gap.
/// * `gap_ext`  – penalty for extending a gap.
/// * `sim_func` – element similarity; defaults to 1 for equal, 0 otherwise.
/// * `qval`     – q-gram tokenization (1 = per element, the default).
#[derive(Debug, Clone, Copy)]
pub struct Gotoh<T> {
    pub qval: usize,
    pub gap_open: f64,
    pub gap_ext: f64,
    pub sim_func: SimFunc<T>,
}

impl<T> Default for Gotoh<T> {
    fn default() -> Self {
        Self {
            qval: 1,
            gap_open: 1.0,
            gap_ext: 0.4,
            sim_func: None,
        }
    }
}

impl<T> Gotoh<T> {
    /// Create a `Gotoh` metric.
    pub fn new(qval: usize, gap_open: f64, gap_ext: f64, sim_func: SimFunc<T>) -> Self {
        Self {
            qval,
            gap_open,
            gap_ext,
            sim_func,
        }
    }

    /// Minimum possible score for the given inputs (`-min_len`).
    pub fn minimum(&self, s1: &[T], s2: &[T]) -> f64 {
        -(s1.len().min(s2.len()) as f64)
    }
}

impl<T: Clone + PartialEq> Base<T> for Gotoh<T> {
    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().min(s2.len()) as f64
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if self.qval <= 1 {
            let sim = self
                .sim_func
                .unwrap_or(|a, b| if a == b { 1.0 } else { 0.0 });
            gotoh_dp(s1, s2, self.gap_open, self.gap_ext, sim)
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            gotoh_dp(
                &g1,
                &g2,
                self.gap_open,
                self.gap_ext,
                |a: &Vec<T>, b: &Vec<T>| {
                    if a == b { 1.0 } else { 0.0 }
                },
            )
        }
    }

    fn distance(&self, s1: &[T], s2: &[T]) -> f64 {
        -self.similarity(s1, s2)
    }

    fn normalized_distance(&self, s1: &[T], s2: &[T]) -> f64 {
        global_normalized_distance(
            self.distance(s1, s2),
            self.minimum(s1, s2),
            self.maximum(s1, s2),
        )
    }

    fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        global_normalized_similarity(
            self.similarity(s1, s2),
            self.minimum(s1, s2),
            self.maximum(s1, s2),
        )
    }
}

/// Smith-Waterman local alignment DP.
///
/// Like [`needleman_wunsch_dp`] but cells are floored at 0 so only the best
/// local region contributes; the bottom-right cell is returned.
fn smith_waterman_dp<U>(s1: &[U], s2: &[U], gap_cost: f64, sim: impl Fn(&U, &U) -> f64) -> f64 {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut dist = vec![vec![0.0f64; len2 + 1]; len1 + 1];
    for i in 1..=len1 {
        for j in 1..=len2 {
            let subst = dist[i - 1][j - 1] + sim(&s1[i - 1], &s2[j - 1]);
            let delete = dist[i - 1][j] - gap_cost;
            let insert = dist[i][j - 1] - gap_cost;
            dist[i][j] = 0.0f64.max(subst).max(delete).max(insert);
        }
    }
    dist[len1][len2]
}

/// Smith-Waterman similarity: local alignment score.
///
/// * `gap_cost` – penalty subtracted for each gap.
/// * `sim_func` – element similarity; defaults to 1 for equal, 0 otherwise.
/// * `qval`     – q-gram tokenization (1 = per element, the default).
///
/// Identical sequences return `maximum` (`min_len`) and empty inputs return 0,
/// mirroring the Python `quick_answer`.
#[derive(Debug, Clone, Copy)]
pub struct SmithWaterman<T> {
    pub qval: usize,
    pub gap_cost: f64,
    pub sim_func: SimFunc<T>,
}

impl<T> Default for SmithWaterman<T> {
    fn default() -> Self {
        Self {
            qval: 1,
            gap_cost: 1.0,
            sim_func: None,
        }
    }
}

impl<T> SmithWaterman<T> {
    /// Create a `SmithWaterman` metric.
    pub fn new(qval: usize, gap_cost: f64, sim_func: SimFunc<T>) -> Self {
        Self {
            qval,
            gap_cost,
            sim_func,
        }
    }
}

impl<T: Clone + PartialEq> Base<T> for SmithWaterman<T> {
    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().min(s2.len()) as f64
    }

    fn similarity(&self, s1: &[T], s2: &[T]) -> f64 {
        if is_ident(s1, s2) {
            return self.maximum(s1, s2);
        }
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }
        if self.qval <= 1 {
            let sim = self
                .sim_func
                .unwrap_or(|a, b| if a == b { 1.0 } else { 0.0 });
            smith_waterman_dp(s1, s2, self.gap_cost, sim)
        } else {
            let g1 = find_ngrams(s1, self.qval);
            let g2 = find_ngrams(s2, self.qval);
            smith_waterman_dp(
                &g1,
                &g2,
                self.gap_cost,
                |a: &Vec<T>, b: &Vec<T>| {
                    if a == b { 1.0 } else { 0.0 }
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    fn assert_close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "expected {b}, got {a}");
    }

    #[test]
    fn test_hamming_python_values() {
        let h = Hamming::default();
        let cases = [
            ("test", "text", 1.0),
            ("test", "tset", 2.0),
            ("test", "qwe", 4.0),
            ("test", "testit", 2.0),
            ("test", "tesst", 2.0),
            ("test", "tet", 2.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(h.distance(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_hamming_truncate() {
        let h = Hamming::new(1, None, true);
        assert_eq!(h.distance(&chars("test"), &chars("qwe")), 3.0);
        assert_eq!(h.distance(&chars("test"), &chars("testit")), 0.0);
    }

    #[test]
    fn test_hamming_edge_cases() {
        let h = Hamming::default();
        assert_eq!(h.distance(&chars(""), &chars("")), 0.0);
        assert_eq!(h.distance(&chars(""), &chars("abc")), 3.0);
        assert_eq!(h.distance(&chars("hello"), &chars("hello")), 0.0);
        assert_eq!(h.distance(&chars("café"), &chars("cafè")), 1.0);
    }

    #[test]
    fn test_hamming_qval() {
        let h = Hamming::new(2, None, false);
        assert_eq!(h.distance(&chars("test"), &chars("text")), 2.0);
    }

    #[test]
    fn test_levenshtein_python_values() {
        let l = Levenshtein::default();
        let cases = [
            ("test", "text", 1.0),
            ("test", "tset", 2.0),
            ("test", "qwe", 4.0),
            ("test", "testit", 2.0),
            ("test", "tesst", 1.0),
            ("test", "tet", 1.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(l.distance(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_levenshtein_edge_cases() {
        let l = Levenshtein::default();
        assert_eq!(l.distance(&chars(""), &chars("")), 0.0);
        assert_eq!(l.distance(&chars(""), &chars("abc")), 3.0);
        assert_eq!(l.distance(&chars("abc"), &chars("")), 3.0);
        assert_eq!(l.distance(&chars("hello"), &chars("hello")), 0.0);
        assert_eq!(l.distance(&chars("café"), &chars("cafè")), 1.0);
    }

    #[test]
    fn test_levenshtein_qval() {
        let l = Levenshtein::new(2, None);
        assert_eq!(l.distance(&chars("test"), &chars("text")), 2.0);
        let l3 = Levenshtein::new(3, None);
        assert_eq!(l3.distance(&chars("hello"), &chars("hxllo")), 2.0);
    }

    #[test]
    fn test_damerau_restricted_python_values() {
        let d = DamerauLevenshtein::default();
        let cases = [
            ("test", "text", 1.0),
            ("test", "tset", 1.0),
            ("test", "qwy", 4.0),
            ("test", "testit", 2.0),
            ("test", "tesst", 1.0),
            ("test", "tet", 1.0),
            ("cat", "hat", 1.0),
            ("Niall", "Neil", 3.0),
            ("aluminum", "Catalan", 7.0),
            ("ATCG", "TAGC", 2.0),
            ("ab", "ba", 1.0),
            ("ab", "cde", 3.0),
            ("ab", "ac", 1.0),
            ("ab", "bc", 2.0),
            ("ab", "bca", 3.0),
            ("abcd", "bdac", 4.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(d.distance(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_damerau_unrestricted_python_values() {
        let d = DamerauLevenshtein::new(1, None, false);
        let cases = [
            ("test", "text", 1.0),
            ("test", "tset", 1.0),
            ("test", "qwy", 4.0),
            ("test", "testit", 2.0),
            ("test", "tesst", 1.0),
            ("test", "tet", 1.0),
            ("cat", "hat", 1.0),
            ("Niall", "Neil", 3.0),
            ("aluminum", "Catalan", 7.0),
            ("ATCG", "TAGC", 2.0),
            ("ab", "ba", 1.0),
            ("ab", "cde", 3.0),
            ("ab", "ac", 1.0),
            ("ab", "bc", 2.0),
            ("ab", "bca", 2.0),
            ("abcd", "bdac", 3.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(d.distance(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_damerau_edge_cases() {
        let d = DamerauLevenshtein::default();
        let u = DamerauLevenshtein::new(1, None, false);
        assert_eq!(d.distance(&chars(""), &chars("")), 0.0);
        assert_eq!(d.distance(&chars(""), &chars("abc")), 3.0);
        assert_eq!(d.distance(&chars("hello"), &chars("hello")), 0.0);
        assert_eq!(d.distance(&chars("café"), &chars("cafè")), 1.0);
        assert_eq!(u.distance(&chars(""), &chars("")), 0.0);
        assert_eq!(u.distance(&chars("ab"), &chars("bca")), 2.0);
    }

    #[test]
    fn test_damerau_qval() {
        let d = DamerauLevenshtein::new(2, None, true);
        assert_eq!(d.distance(&chars("test"), &chars("text")), 2.0);
    }

    #[test]
    fn test_custom_test_func() {
        fn ignore_case(a: &char, b: &char) -> bool {
            a.to_lowercase().next() == b.to_lowercase().next()
        }
        let h = Hamming::new(1, Some(ignore_case), false);
        assert_eq!(h.distance(&chars("ABC"), &chars("abc")), 0.0);
        let l = Levenshtein::new(1, Some(ignore_case));
        assert_eq!(l.distance(&chars("ABC"), &chars("abc")), 0.0);
    }

    #[test]
    fn test_normalization_invariants() {
        let algs: Vec<Box<dyn Base<char>>> = vec![
            Box::new(Hamming::default()),
            Box::new(Levenshtein::default()),
            Box::new(DamerauLevenshtein::default()),
            Box::new(DamerauLevenshtein::new(1, None, false)),
        ];
        for alg in algs {
            let d = alg.normalized_distance(&chars("abcde"), &chars("abxde"));
            let s = alg.normalized_similarity(&chars("abcde"), &chars("abxde"));
            assert!((0.0..=1.0).contains(&d));
            assert!((0.0..=1.0).contains(&s));
            assert!((s + d - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_jaro_python_values() {
        let j = Jaro::default();
        let cases = [
            ("hello", "haloa", 0.7333333333333334),
            ("fly", "ant", 0.0),
            ("frog", "fog", 0.9166666666666666),
            ("ATCG", "TAGC", 0.8333333333333334),
            ("MARTHA", "MARHTA", 0.944444444),
            ("DWAYNE", "DUANE", 0.822222222),
            ("DIXON", "DICKSONX", 0.7666666666666666),
            (
                "Sint-Pietersplein 6, 9000 Gent",
                "Test 10, 1010 Brussel",
                0.5182539682539683,
            ),
        ];
        for (a, b, expected) in cases {
            assert_close(j.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_jaro_winkler_python_values() {
        let j = JaroWinkler::default();
        let cases = [
            ("elephant", "hippo", 0.44166666666666665),
            ("fly", "ant", 0.0),
            ("frog", "fog", 0.925),
            ("MARTHA", "MARHTA", 0.9611111111111111),
            ("DWAYNE", "DUANE", 0.84),
            ("DIXON", "DICKSONX", 0.8133333333333332),
            ("duck donald", "duck daisy", 0.867272727272),
        ];
        for (a, b, expected) in cases {
            assert_close(j.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_jaro_edge_cases() {
        let j = Jaro::default();
        assert_eq!(j.similarity(&chars(""), &chars("")), 1.0);
        assert_eq!(j.similarity(&chars(""), &chars("abc")), 0.0);
        assert_eq!(j.similarity(&chars("hello"), &chars("hello")), 1.0);
        let jw = JaroWinkler::default();
        assert_eq!(jw.similarity(&chars(""), &chars("")), 1.0);
        assert_eq!(jw.similarity(&chars(""), &chars("abc")), 0.0);
        assert_eq!(jw.similarity(&chars("hello"), &chars("hello")), 1.0);
    }

    #[test]
    fn test_jaro_winkler_matches_jaro_when_disabled() {
        let j = Jaro::default();
        let jw = JaroWinkler::new(false, false, 1);
        assert_close(
            jw.similarity(&chars("MARTHA"), &chars("MARHTA")),
            j.similarity(&chars("MARTHA"), &chars("MARHTA")),
        );
    }

    #[test]
    fn test_strcmp95_python_values() {
        let s = StrCmp95::default();
        let cases = [
            ("MARTHA", "MARHTA", 0.9611111111111111),
            ("DWAYNE", "DUANE", 0.873),
            ("DIXON", "DICKSONX", 0.839333333),
            ("TEST", "TEXT", 0.9066666666666666),
        ];
        for (a, b, expected) in cases {
            assert_close(s.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_strcmp95_edge_cases() {
        let s = StrCmp95::default();
        assert_eq!(s.similarity(&chars(""), &chars("")), 1.0);
        assert_eq!(s.similarity(&chars(""), &chars("ABC")), 0.0);
        assert_eq!(s.similarity(&chars("hello"), &chars("HELLO")), 1.0);
        assert_eq!(s.similarity(&chars("  hello  "), &chars("hello")), 1.0);
    }

    #[test]
    fn test_mlipns_python_values() {
        let m = MLIPNS::default();
        let cases = [
            ("", "", 1.0),
            ("a", "", 0.0),
            ("", "a", 0.0),
            ("a", "a", 1.0),
            ("ab", "a", 1.0),
            ("abc", "abc", 1.0),
            ("abc", "abcde", 1.0),
            ("abcg", "abcdeg", 1.0),
            ("abcg", "abcdefg", 0.0),
            ("Tomato", "Tamato", 1.0),
            ("ato", "Tam", 1.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(m.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_normalization_invariants_part3() {
        let algs: Vec<Box<dyn Base<char>>> = vec![
            Box::new(Jaro::default()),
            Box::new(JaroWinkler::default()),
            Box::new(StrCmp95::default()),
            Box::new(MLIPNS::default()),
        ];
        for alg in algs {
            let d = alg.normalized_distance(&chars("abcde"), &chars("abxde"));
            let s = alg.normalized_similarity(&chars("abcde"), &chars("abxde"));
            assert!((0.0..=1.0).contains(&d));
            assert!((0.0..=1.0).contains(&s));
            assert!((s + d - 1.0).abs() < 1e-9);
        }
    }

    // ─── Part 4: NeedlemanWunsch / Gotoh / SmithWaterman ─────────────────────

    fn sim_ident(a: &char, b: &char) -> f64 {
        if a == b { 1.0 } else { -1.0 }
    }

    /// Lookup table matching Python's `Matrix(NW_MATRIX, symmetric=True)` with
    /// the default `match_cost = 1` and `mismatch_cost = 0`.
    fn sim_matrix(a: &char, b: &char) -> f64 {
        const NW_MATRIX: [((char, char), f64); 10] = [
            (('A', 'A'), 10.0),
            (('G', 'G'), 7.0),
            (('C', 'C'), 9.0),
            (('T', 'T'), 8.0),
            (('A', 'G'), -1.0),
            (('A', 'C'), -3.0),
            (('A', 'T'), -4.0),
            (('G', 'C'), -5.0),
            (('G', 'T'), -3.0),
            (('C', 'T'), 0.0),
        ];
        for ((x, y), v) in NW_MATRIX {
            if (*a == x && *b == y) || (*a == y && *b == x) {
                return v;
            }
        }
        if a == b { 1.0 } else { 0.0 }
    }

    #[test]
    fn test_needleman_wunsch_matrix() {
        let nw = NeedlemanWunsch::new(1, 5.0, Some(sim_matrix));
        assert_close(
            nw.similarity(&chars("AGACTAGTTAC"), &chars("CGAGACGT")),
            16.0,
        );
    }

    #[test]
    fn test_needleman_wunsch_ident() {
        let nw = NeedlemanWunsch::new(1, 1.0, Some(sim_ident));
        assert_close(nw.similarity(&chars("GATTACA"), &chars("GCATGCU")), 0.0);
    }

    #[test]
    fn test_needleman_wunsch_ident_gap5() {
        let nw = NeedlemanWunsch::new(1, 5.0, Some(sim_ident));
        let cases = [
            ("CGATATCAG", "TGACGSTGC", -5.0),
            ("AGACTAGTTAC", "TGACGSTGC", -7.0),
            ("AGACTAGTTAC", "CGAGACGT", -15.0),
        ];
        for (a, b, expected) in cases {
            assert_close(nw.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_needleman_wunsch_default_values() {
        let nw = NeedlemanWunsch::<char>::default();
        assert_close(nw.similarity(&chars("abc"), &chars("abc")), 3.0);
        assert_close(nw.distance(&chars("abc"), &chars("abc")), -3.0);
        assert_close(nw.normalized_distance(&chars("abc"), &chars("abc")), 0.0);
        assert_close(nw.normalized_similarity(&chars("abc"), &chars("abc")), 1.0);

        assert_close(nw.similarity(&chars("abc"), &chars("abx")), 2.0);
        assert_close(nw.distance(&chars("abc"), &chars("abx")), -2.0);
        assert_close(
            nw.normalized_distance(&chars("abc"), &chars("abx")),
            0.16666666666666666,
        );
        assert_close(
            nw.normalized_similarity(&chars("abc"), &chars("abx")),
            0.8333333333333334,
        );

        assert_close(nw.similarity(&chars("a"), &chars("")), -1.0);
        assert_close(nw.distance(&chars("a"), &chars("")), 1.0);
        assert_close(nw.normalized_distance(&chars("a"), &chars("")), 1.0);
        assert_close(nw.normalized_similarity(&chars("a"), &chars("")), 0.0);

        assert_close(nw.similarity(&chars(""), &chars("")), 0.0);
        assert_close(nw.distance(&chars(""), &chars("")), 0.0);
        assert_close(nw.normalized_distance(&chars(""), &chars("")), 0.0);
        assert_close(nw.normalized_similarity(&chars(""), &chars("")), 1.0);
    }

    #[test]
    fn test_needleman_wunsch_qval() {
        let nw = NeedlemanWunsch::new(2, 1.0, None);
        assert_close(nw.similarity(&chars("hello"), &chars("hello")), 4.0);
        assert_close(nw.similarity(&chars("hello"), &chars("helxo")), 2.0);
    }

    #[test]
    fn test_gotoh_ident() {
        let g = Gotoh::new(1, 1.0, 1.0, Some(sim_ident));
        assert_close(g.similarity(&chars("GATTACA"), &chars("GCATGCU")), 0.0);
    }

    #[test]
    fn test_gotoh_ident_gap05() {
        let g = Gotoh::new(1, 1.0, 0.5, Some(sim_ident));
        let cases = [
            ("GATTACA", "GCATGCU", 0.0),
            ("AGACTAGTTAC", "TGACGSTGC", 1.5),
            ("AGACTAGTTAC", "CGAGACGT", 1.0),
        ];
        for (a, b, expected) in cases {
            assert_close(g.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_gotoh_ident_gap5() {
        let g = Gotoh::new(1, 5.0, 5.0, Some(sim_ident));
        assert_close(
            g.similarity(&chars("AGACTAGTTAC"), &chars("CGAGACGT")),
            -15.0,
        );
    }

    #[test]
    fn test_gotoh_default_values() {
        let g = Gotoh::<char>::default();
        assert_close(g.similarity(&chars("abc"), &chars("abc")), 3.0);
        assert_close(g.distance(&chars("abc"), &chars("abc")), -3.0);
        assert_close(g.normalized_distance(&chars("abc"), &chars("abc")), 0.0);
        assert_close(g.normalized_similarity(&chars("abc"), &chars("abc")), 1.0);

        assert_close(g.similarity(&chars("abc"), &chars("abx")), 2.0);
        assert_close(g.distance(&chars("abc"), &chars("abx")), -2.0);
        assert_close(
            g.normalized_distance(&chars("abc"), &chars("abx")),
            0.16666666666666666,
        );
        assert_close(
            g.normalized_similarity(&chars("abc"), &chars("abx")),
            0.8333333333333334,
        );

        assert_close(g.similarity(&chars(""), &chars("")), 0.0);
        assert_close(g.distance(&chars(""), &chars("")), 0.0);
        assert_close(g.normalized_distance(&chars(""), &chars("")), 0.0);
        assert_close(g.normalized_similarity(&chars(""), &chars("")), 1.0);
    }

    #[test]
    fn test_gotoh_ident_values() {
        let g = Gotoh::new(1, 1.0, 1.0, Some(sim_ident));
        assert_close(g.similarity(&chars("abc"), &chars("abc")), 3.0);
        assert_close(g.distance(&chars("abc"), &chars("abc")), -3.0);
        assert_close(g.normalized_distance(&chars("abc"), &chars("abc")), 0.0);
        assert_close(g.normalized_similarity(&chars("abc"), &chars("abc")), 1.0);

        assert_close(g.similarity(&chars("abc"), &chars("abx")), 1.0);
        assert_close(g.distance(&chars("abc"), &chars("abx")), -1.0);
        assert_close(
            g.normalized_distance(&chars("abc"), &chars("abx")),
            0.3333333333333333,
        );
        assert_close(
            g.normalized_similarity(&chars("abc"), &chars("abx")),
            0.6666666666666666,
        );
    }

    #[test]
    fn test_gotoh_empty_deviation() {
        let g = Gotoh::<char>::default();
        assert_close(g.similarity(&chars(""), &chars("a")), -1.0);
        assert_close(g.similarity(&chars("a"), &chars("")), -1.0);
        assert_close(g.similarity(&chars(""), &chars("ab")), -1.4);
        assert_close(g.similarity(&chars("ab"), &chars("")), -1.4);
    }

    #[test]
    fn test_gotoh_qval() {
        let g = Gotoh::new(2, 1.0, 0.4, None);
        assert_close(g.similarity(&chars("hello"), &chars("hello")), 4.0);
        assert_close(g.similarity(&chars("hello"), &chars("helxo")), 2.0);
    }

    #[test]
    fn test_smith_waterman_matrix() {
        let sw = SmithWaterman::new(1, 5.0, Some(sim_matrix));
        assert_close(
            sw.similarity(&chars("AGACTAGTTAC"), &chars("CGAGACGT")),
            26.0,
        );
    }

    #[test]
    fn test_smith_waterman_ident() {
        let sw = SmithWaterman::new(1, 1.0, Some(sim_ident));
        assert_close(sw.similarity(&chars("GATTACA"), &chars("GCATGCU")), 0.0);
    }

    #[test]
    fn test_smith_waterman_ident_gap5() {
        let sw = SmithWaterman::new(1, 5.0, Some(sim_ident));
        let cases = [
            ("CGATATCAG", "TGACGSTGC", 0.0),
            ("AGACTAGTTAC", "TGACGSTGC", 1.0),
            ("AGACTAGTTAC", "CGAGACGT", 0.0),
        ];
        for (a, b, expected) in cases {
            assert_close(sw.similarity(&chars(a), &chars(b)), expected);
        }
    }

    #[test]
    fn test_smith_waterman_default_values() {
        let sw = SmithWaterman::<char>::default();
        assert_close(sw.similarity(&chars("abc"), &chars("abc")), 3.0);
        assert_close(sw.distance(&chars("abc"), &chars("abc")), 0.0);
        assert_close(sw.normalized_distance(&chars("abc"), &chars("abc")), 0.0);
        assert_close(sw.normalized_similarity(&chars("abc"), &chars("abc")), 1.0);

        assert_close(sw.similarity(&chars("abc"), &chars("abx")), 2.0);
        assert_close(sw.distance(&chars("abc"), &chars("abx")), 1.0);
        assert_close(
            sw.normalized_distance(&chars("abc"), &chars("abx")),
            0.3333333333333333,
        );
        assert_close(
            sw.normalized_similarity(&chars("abc"), &chars("abx")),
            0.6666666666666667,
        );

        assert_close(sw.similarity(&chars("a"), &chars("")), 0.0);
        assert_close(sw.distance(&chars("a"), &chars("")), 0.0);
        assert_close(sw.normalized_distance(&chars("a"), &chars("")), 0.0);
        assert_close(sw.normalized_similarity(&chars("a"), &chars("")), 1.0);

        assert_close(sw.similarity(&chars(""), &chars("")), 0.0);
        assert_close(sw.distance(&chars(""), &chars("")), 0.0);
        assert_close(sw.normalized_distance(&chars(""), &chars("")), 0.0);
        assert_close(sw.normalized_similarity(&chars(""), &chars("")), 1.0);
    }

    #[test]
    fn test_smith_waterman_qval() {
        let sw = SmithWaterman::new(2, 1.0, None);
        assert_close(sw.similarity(&chars("hello"), &chars("helxo")), 2.0);
    }
}
