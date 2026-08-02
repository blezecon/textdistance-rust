//! Phonetic distance algorithms (Person 1 assignment).
//!
//! Ports the Python `textdistance.algorithms.phonetic` module: `MRA` and
//! `Editex`.

use crate::base::{Base, is_ident};

// ─── MRA ─────────────────────────────────────────────────────────────────────

/// Match Rating Approach (Western Airlines surname comparison) rating.
///
/// Mirrors Python's `textdistance.MRA`, a `_BaseSimilarity`: the returned
/// value is a similarity score in `[0, maximum]`. Operates on uppercase ASCII
/// `char` sequences; vowels (`AEIOU`) are removed from every position except
/// the first, consecutive repeats are collapsed, and codes longer than 6
/// characters are truncated to `first3 + last3`.
///
/// Deviation: characters are upper-cased with `char::to_uppercase`'s first
/// code point, so lowercase glyphs that uppercase to several characters (e.g.
/// `ß` → `SS`) collapse to a single character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MRA;

impl MRA {
    /// Create an `MRA` metric.
    pub fn new() -> Self {
        Self
    }
}

/// Build the MRA code for one word: uppercase, keep the first character and
/// drop vowels from the rest, collapse consecutive repeats, then truncate
/// codes longer than 6 characters to `first3 + last3`. Mirrors Python's
/// `MRA._calc_mra`.
fn mra_calc(word: &[char]) -> Vec<char> {
    if word.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(word.len());
    for (i, &c) in word.iter().enumerate() {
        let c = c.to_uppercase().next().unwrap_or(c);
        if i == 0
            || (c != 'A'
                && c != 'E'
                && c != 'I'
                && c != 'O'
                && c != 'U'
                && result.last() != Some(&c))
        {
            result.push(c);
        }
    }
    if result.len() > 6 {
        let mut truncated: Vec<char> = result[..3].to_vec();
        truncated.extend_from_slice(&result[result.len() - 3..]);
        return truncated;
    }
    result
}

/// MRA comparison rating over already-coded sequences.
///
/// Mirrors Python's `MRA.__call__`: after a length-gap bail-out, runs
/// `count` passes where every position whose characters agree across all
/// sequences is removed from all of them. The rating is
/// `max_length - max(remaining lengths)`.
fn mra_compare(seqs: &mut [Vec<char>]) -> usize {
    let count = seqs.len();
    let max_length = seqs.iter().map(Vec::len).max().unwrap_or(0);
    let min_length = seqs.iter().map(Vec::len).min().unwrap_or(0);
    if max_length.abs_diff(min_length) > count {
        return 0;
    }
    for _ in 0..count {
        let minlen = seqs.iter().map(Vec::len).min().unwrap_or(0);
        let keep: Vec<bool> = (0..minlen)
            .map(|i| {
                let first = seqs[0][i];
                seqs[1..].iter().any(|s| s[i] != first)
            })
            .collect();
        for seq in seqs.iter_mut() {
            let mut filtered: Vec<char> =
                (0..minlen).filter(|&i| keep[i]).map(|i| seq[i]).collect();
            filtered.extend_from_slice(&seq[minlen..]);
            *seq = filtered;
        }
    }
    let max_remaining = seqs.iter().map(Vec::len).max().unwrap_or(0);
    max_length - max_remaining
}

impl Base<char> for MRA {
    fn maximum(&self, s1: &[char], s2: &[char]) -> f64 {
        mra_calc(s1).len().max(mra_calc(s2).len()) as f64
    }

    fn similarity(&self, s1: &[char], s2: &[char]) -> f64 {
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }
        mra_compare(&mut [mra_calc(s1), mra_calc(s2)]) as f64
    }
}

// ─── Editex ──────────────────────────────────────────────────────────────────

/// Letter classes used by Editex, as in Python's `Editex.groups`.
const EDITEX_GROUPS: [&[u8]; 10] = [
    b"AEIOUY", b"BP", b"CKQ", b"DT", b"LR", b"MN", b"GJ", b"FPV", b"SXZ", b"CSZ",
];

/// Letters not present in any group, as in Python's `Editex.ungrouped`.
const EDITEX_UNGROUPED: &[u8] = b"HW";

/// Editex phonetic edit distance (distance-first).
///
/// Mirrors Python's `textdistance.Editex`: a letter-class edit distance where
/// the cost of aligning two letters depends on whether they are equal, fall
/// into the same phonetic class, or are unrelated. The leading-space padding
/// and row/column initialization are replicated exactly, including `local`
/// mode (which skips the first-column initialization).
///
/// Deviation: characters are upper-cased per `char` (first code point), so
/// lowercase glyphs that uppercase to several characters (e.g. `ß`) collapse
/// to one. The `external` flag is dropped: the port never delegates to an
/// external library and always returns the internal implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Editex {
    pub local: bool,
    pub match_cost: usize,
    pub group_cost: usize,
    pub mismatch_cost: usize,
}

impl Default for Editex {
    fn default() -> Self {
        Self {
            local: false,
            match_cost: 0,
            group_cost: 1,
            mismatch_cost: 2,
        }
    }
}

impl Editex {
    /// Create an `Editex` metric.
    ///
    /// Costs are normalized like Python's `__init__` so that
    /// `match_cost <= group_cost <= mismatch_cost`.
    pub fn new(local: bool, match_cost: usize, group_cost: usize, mismatch_cost: usize) -> Self {
        let group_cost = group_cost.max(match_cost);
        let mismatch_cost = mismatch_cost.max(group_cost);
        Self {
            local,
            match_cost,
            group_cost,
            mismatch_cost,
        }
    }

    /// Replacement cost of aligning `a` with `b`: the `match_cost` when equal,
    /// the `mismatch_cost` when either letter is ungrouped or from different
    /// classes, and the `group_cost` when both share a letter class.
    fn r_cost(&self, a: char, b: char) -> usize {
        if a == b {
            return self.match_cost;
        }
        let a = a as u8;
        let b = b as u8;
        let grouped = |c: u8| EDITEX_GROUPS.iter().any(|g| g.contains(&c));
        if !grouped(a) || !grouped(b) {
            return self.mismatch_cost;
        }
        if EDITEX_GROUPS
            .iter()
            .any(|g| g.contains(&a) && g.contains(&b))
        {
            return self.group_cost;
        }
        self.mismatch_cost
    }

    /// Deletion cost of removing `curr` after `prev`, mirroring Python's
    /// `Editex.d_cost`: aligning the current character with an ungrouped
    /// neighbor costs the `group_cost`.
    fn d_cost(&self, prev: char, curr: char) -> usize {
        if prev != curr && EDITEX_UNGROUPED.contains(&(prev as u8)) {
            return self.group_cost;
        }
        self.r_cost(prev, curr)
    }
}

impl Base<char> for Editex {
    fn maximum(&self, s1: &[char], s2: &[char]) -> f64 {
        s1.len().max(s2.len()) as f64 * self.mismatch_cost as f64
    }

    fn distance(&self, s1: &[char], s2: &[char]) -> f64 {
        if is_ident(s1, s2) {
            return 0.0;
        }
        if s1.is_empty() || s2.is_empty() {
            return self.maximum(s1, s2);
        }

        let max_length = self.maximum(s1, s2);
        let s1: Vec<char> = std::iter::once(' ')
            .chain(s1.iter().map(|c| c.to_uppercase().next().unwrap_or(*c)))
            .collect();
        let s2: Vec<char> = std::iter::once(' ')
            .chain(s2.iter().map(|c| c.to_uppercase().next().unwrap_or(*c)))
            .collect();
        let len_s1 = s1.len() - 1;
        let len_s2 = s2.len() - 1;

        let mut d_mat = vec![vec![0usize; len_s2 + 1]; len_s1 + 1];

        if !self.local {
            for i in 1..=len_s1 {
                d_mat[i][0] = d_mat[i - 1][0] + self.d_cost(s1[i - 1], s1[i]);
            }
        }
        for j in 1..=len_s2 {
            d_mat[0][j] = d_mat[0][j - 1] + self.d_cost(s2[j - 1], s2[j]);
        }

        for i in 1..=len_s1 {
            let cs1_prev = s1[i - 1];
            let cs1_curr = s1[i];
            for j in 1..=len_s2 {
                let cs2_prev = s2[j - 1];
                let cs2_curr = s2[j];
                d_mat[i][j] = (d_mat[i - 1][j] + self.d_cost(cs1_prev, cs1_curr))
                    .min(d_mat[i][j - 1] + self.d_cost(cs2_prev, cs2_curr))
                    .min(d_mat[i - 1][j - 1] + self.r_cost(cs1_curr, cs2_curr));
            }
        }

        (d_mat[len_s1][len_s2] as f64).min(max_length)
    }
}

// ─── tests ───────────────────────────────────────────────────────────────────

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
    fn test_mra_python_values() {
        let m = MRA;
        let cases = [
            ("", "", 0.0),
            ("a", "", 0.0),
            ("", "a", 0.0),
            ("Byrne", "Boern", 1.0),
            ("Smith", "Smyth", 2.0),
            ("Byrne", "Smith", 0.0),
            ("Smith", "Smythe", 2.0),
            ("bryne", "boern", 2.0),
            ("MARTHA", "MARHTA", 2.0),
            ("abc", "abc", 3.0),
            ("aaaaaaaa", "aaaaaaaa", 1.0),
            ("abcdef", "abf", 2.0),
            ("Washington", "Wasington", 5.0),
            ("washingt", "washignt", 4.0),
            ("aaaaaaabbbbbbcccccc", "aaaaaaabbbbbbdddddd", 2.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(m.similarity(&chars(a), &chars(b)), expected, "{a} vs {b}");
        }
    }

    #[test]
    fn test_mra_maximum_and_derived() {
        let m = MRA;
        assert_eq!(m.maximum(&chars(""), &chars("")), 0.0);
        assert_eq!(m.maximum(&chars("a"), &chars("")), 1.0);
        assert_eq!(m.maximum(&chars("Byrne"), &chars("Boern")), 4.0);
        assert_eq!(m.maximum(&chars("Smith"), &chars("Smyth")), 5.0);
        assert_eq!(m.maximum(&chars("Washington"), &chars("Wasington")), 6.0);
        assert_eq!(m.distance(&chars("abcdef"), &chars("abf")), 3.0);
        assert_eq!(m.distance(&chars("abc"), &chars("abc")), 0.0);
        assert_close(m.normalized_distance(&chars("abcdef"), &chars("abf")), 0.6);
        assert_close(
            m.normalized_similarity(&chars("abcdef"), &chars("abf")),
            0.4,
        );
    }

    #[test]
    fn test_mra_calc_truncation() {
        assert_eq!(mra_calc(&chars("Washington")), chars("WSHGTN"));
        assert_eq!(mra_calc(&chars("AAAAAAABBBBBBCCCCCC")), chars("ABC"));
        assert_eq!(mra_calc(&chars("")), Vec::<char>::new());
        assert_eq!(mra_calc(&chars("A")), chars("A"));
    }

    #[test]
    fn test_editex_python_values() {
        let e = Editex::default();
        let cases = [
            ("", "", 0.0),
            ("nelson", "", 12.0),
            ("", "neilsen", 14.0),
            ("ab", "a", 2.0),
            ("ab", "c", 4.0),
            ("nelson", "neilsen", 2.0),
            ("neilsen", "nelson", 2.0),
            ("niall", "neal", 1.0),
            ("neal", "niall", 1.0),
            ("niall", "nihal", 2.0),
            ("nihal", "niall", 2.0),
            ("neal", "nihl", 3.0),
            ("nihl", "neal", 3.0),
            ("cat", "hat", 2.0),
            ("Niall", "Neil", 2.0),
            ("aluminum", "Catalan", 12.0),
            ("ATCG", "TAGC", 6.0),
            ("ALIE", "ALI", 1.0),
            ("", "MARTHA", 12.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(e.distance(&chars(a), &chars(b)), expected, "{a} vs {b}");
        }
    }

    #[test]
    fn test_editex_local() {
        let e = Editex::new(true, 0, 1, 2);
        let cases = [
            ("", "", 0.0),
            ("nelson", "", 12.0),
            ("", "neilsen", 14.0),
            ("ab", "a", 2.0),
            ("ab", "c", 2.0),
            ("nelson", "neilsen", 2.0),
            ("neilsen", "nelson", 2.0),
            ("niall", "neal", 1.0),
            ("neal", "niall", 1.0),
            ("niall", "nihal", 2.0),
            ("nihal", "niall", 2.0),
            ("neal", "nihl", 3.0),
            ("nihl", "neal", 3.0),
        ];
        for (a, b, expected) in cases {
            assert_eq!(e.distance(&chars(a), &chars(b)), expected, "{a} vs {b}");
        }
    }

    #[test]
    fn test_editex_with_params() {
        let cases = [
            (false, 2, 1, 2, 12.0),
            (false, 4, 1, 2, 24.0),
            (true, 0, 1, 2, 3.0),
            (true, 0, 2, 2, 4.0),
            (true, 0, 1, 4, 5.0),
        ];
        for (local, match_cost, group_cost, mismatch_cost, expected) in cases {
            let e = Editex::new(local, match_cost, group_cost, mismatch_cost);
            assert_eq!(
                e.distance(&chars("MARTHA"), &chars("MARHTA")),
                expected,
                "{local} {match_cost} {group_cost} {mismatch_cost}"
            );
        }
    }

    #[test]
    fn test_editex_maximum_and_derived() {
        let e = Editex::default();
        assert_eq!(e.maximum(&chars("nelson"), &chars("")), 12.0);
        assert_eq!(e.maximum(&chars("niall"), &chars("neal")), 10.0);
        assert_eq!(e.similarity(&chars("niall"), &chars("neal")), 9.0);
        assert_close(e.normalized_distance(&chars("niall"), &chars("neal")), 0.1);
        assert_close(
            e.normalized_similarity(&chars("niall"), &chars("neal")),
            0.9,
        );
        assert_eq!(e.distance(&chars(""), &chars("")), 0.0);
        assert_eq!(e.distance(&chars("abc"), &chars("abc")), 0.0);
    }

    #[test]
    fn test_editex_ungrouped_boundary() {
        // 'W'/'H' are ungrouped: aligning a letter with an ungrouped neighbor
        // costs the group cost rather than the mismatch cost.
        let e = Editex::default();
        assert_eq!(e.distance(&chars("what"), &chars("hat")), 2.0);
        assert_eq!(e.distance(&chars("awhile"), &chars("while")), 2.0);
    }
}