//! Compression-based distance algorithms (Person 2 assignment)
//!
//! Implements: RLENCD, BWTRLENCD, SqrtNCD, EntropyNCD, ArithNCD, BZ2NCD, LZMANCD, ZLIBNCD
//!
//! All are based on the Normalized Compression Distance (NCD) formula:
//!   NCD(x, y) = (C(xy) - min(C(x), C(y))) / max(C(x), C(y))
//!
//! where C(·) is the compressed size function supplied by each algorithm.

use std::collections::HashMap;
use std::io::Write;

use bzip2::write::BzEncoder;
use bzip2::Compression as BzCompression;
use flate2::write::ZlibEncoder;
use flate2::Compression as ZlibCompression;
use xz2::write::XzEncoder;

// ─── NCD trait ───────────────────────────────────────────────────────────────

/// Trait that every NCD-variant must implement.
///
/// The NCD formula used here matches the Python `_NCDBase.__call__`:
///   * Try every permutation of the concatenated sequences, take the minimum compressed size.
///   * `NCD = (concat_len - min_individual * (n-1)) / max_individual`
///   * `maximum` is always 1.
trait NcdBase {
    /// Return the compressed size of `data`.
    fn get_size(&self, data: &[u8]) -> f64;

    /// Compute the NCD for two byte-slice sequences.
    fn ncd(&self, s1: &[u8], s2: &[u8]) -> f64 {
        // For 2 sequences only two permutations: s1+s2 and s2+s1
        let mut ab = Vec::with_capacity(s1.len() + s2.len());
        ab.extend_from_slice(s1);
        ab.extend_from_slice(s2);

        let mut ba = Vec::with_capacity(s1.len() + s2.len());
        ba.extend_from_slice(s2);
        ba.extend_from_slice(s1);

        let concat_len = self.get_size(&ab).min(self.get_size(&ba));

        let c1 = self.get_size(s1);
        let c2 = self.get_size(s2);
        let max_len = c1.max(c2);

        if max_len == 0.0 {
            return 0.0;
        }
        (concat_len - c1.min(c2)) / max_len
    }
}

// ─── RLENCD ──────────────────────────────────────────────────────────────────

/// Run-Length Encoding NCD.
///
/// The Python implementation tokenises sequences via `_get_sequences` then groups
/// consecutive identical elements.  Runs of length 1 are kept as-is; runs of 2 are
/// doubled; runs of 3+ are encoded as `"{n}{element}"`.  `_get_size` = `len(compressed_string)`.
///
/// <https://en.wikipedia.org/wiki/Run-length_encoding>
#[derive(Debug, Clone, Copy, Default)]
pub struct Rlencd {
    pub qval: usize,
}

impl Rlencd {
    pub fn new(qval: usize) -> Self {
        Self { qval }
    }

    fn compress_chars(data: &[u8]) -> String {
        let mut out = String::new();
        let mut i = 0;
        while i < data.len() {
            let ch = data[i];
            let mut count = 1usize;
            while i + count < data.len() && data[i + count] == ch {
                count += 1;
            }
            if count > 2 {
                out.push_str(&count.to_string());
                out.push(ch as char);
            } else if count == 1 {
                out.push(ch as char);
            } else {
                // count == 2
                out.push(ch as char);
                out.push(ch as char);
            }
            i += count;
        }
        out
    }
}

impl NcdBase for Rlencd {
    fn get_size(&self, data: &[u8]) -> f64 {
        Self::compress_chars(data).len() as f64
    }
}

impl Rlencd {
    /// Compute RLENCD distance between two string slices.
    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        self.ncd(s1.as_bytes(), s2.as_bytes())
    }

    /// RLENCD maximum is always 1.
    pub fn maximum(&self) -> f64 {
        1.0
    }

    /// Normalized distance = distance (already 0..1).
    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }

    /// Normalized similarity = 1 - normalized_distance.
    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

// ─── BWTRLENCD ───────────────────────────────────────────────────────────────

/// Burrows-Wheeler Transform + RLE NCD.
///
/// Applies the BWT (sort all rotations, take last column) then delegates to RLENCD.
/// The Python implementation appends a `terminator` (`\0` by default) to the data,
/// sorts all rotations, and extracts the last character of each rotation.
///
/// <https://en.wikipedia.org/wiki/Burrows%E2%80%93Wheeler_transform>
#[derive(Debug, Clone)]
pub struct Bwtrlencd {
    pub terminator: char,
}

impl Default for Bwtrlencd {
    fn default() -> Self {
        Self { terminator: '\0' }
    }
}

impl Bwtrlencd {
    pub fn new(terminator: char) -> Self {
        Self { terminator }
    }

    fn bwt_compress_bytes(&self, data: &[u8]) -> Vec<u8> {
        let mut s: Vec<u8> = data.to_vec();
        let term = self.terminator as u8;

        if s.is_empty() {
            return vec![term];
        }
        if !s.contains(&term) {
            s.push(term);
        }
        let n = s.len();
        let mut rotations: Vec<usize> = (0..n).collect();
        rotations.sort_by(|&a, &b| {
            for i in 0..n {
                let ca = s[(a + i) % n];
                let cb = s[(b + i) % n];
                if ca != cb {
                    return ca.cmp(&cb);
                }
            }
            std::cmp::Ordering::Equal
        });
        let bwt: Vec<u8> = rotations.iter().map(|&i| s[(i + n - 1) % n]).collect();

        // Now apply RLE on the BWT result
        Rlencd::compress_chars(&bwt).into_bytes()
    }
}

impl NcdBase for Bwtrlencd {
    fn get_size(&self, data: &[u8]) -> f64 {
        self.bwt_compress_bytes(data).len() as f64
    }
}

impl Bwtrlencd {
    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        self.ncd(s1.as_bytes(), s2.as_bytes())
    }

    pub fn maximum(&self) -> f64 {
        1.0
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

// ─── SqrtNCD ─────────────────────────────────────────────────────────────────

/// Square Root NCD.
///
/// The "compressed size" of a sequence is the sum of √count for each unique element,
/// where count is that element's frequency.
///
/// <https://en.wikipedia.org/wiki/Normalized_compression_distance>
#[derive(Debug, Clone, Copy, Default)]
pub struct SqrtNcd {
    pub qval: usize,
}

impl SqrtNcd {
    pub fn new(qval: usize) -> Self {
        Self { qval }
    }
}

impl NcdBase for SqrtNcd {
    fn get_size(&self, data: &[u8]) -> f64 {
        let mut counts: HashMap<u8, usize> = HashMap::new();
        for &b in data {
            *counts.entry(b).or_insert(0) += 1;
        }
        counts.values().map(|&c| (c as f64).sqrt()).sum()
    }
}

impl SqrtNcd {
    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        self.ncd(s1.as_bytes(), s2.as_bytes())
    }

    pub fn maximum(&self) -> f64 {
        1.0
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

// ─── EntropyNCD ──────────────────────────────────────────────────────────────

/// Entropy-based NCD.
///
/// The "compressed size" is `coef + Shannon_entropy`, where entropy is computed as
/// `-Σ p·log_base(p)` with `p = count / total`.
///
/// Parameters: `qval=1`, `coef=1`, `base=2`.
///
/// <https://en.wikipedia.org/wiki/Entropy_(information_theory)>
#[derive(Debug, Clone, Copy)]
pub struct EntropyNcd {
    pub qval: usize,
    pub coef: f64,
    pub base: f64,
}

impl Default for EntropyNcd {
    fn default() -> Self {
        Self {
            qval: 1,
            coef: 1.0,
            base: 2.0,
        }
    }
}

impl EntropyNcd {
    pub fn new(qval: usize, coef: f64, base: f64) -> Self {
        Self { qval, coef, base }
    }

    fn entropy_of(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let total = data.len() as f64;
        let mut counts: HashMap<u8, usize> = HashMap::new();
        for &b in data {
            *counts.entry(b).or_insert(0) += 1;
        }
        let mut entropy = 0.0f64;
        for &c in counts.values() {
            let p = c as f64 / total;
            entropy -= p * p.log(self.base);
        }
        entropy.max(0.0)
    }
}

impl NcdBase for EntropyNcd {
    fn get_size(&self, data: &[u8]) -> f64 {
        self.coef + self.entropy_of(data)
    }
}

impl EntropyNcd {
    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        self.ncd(s1.as_bytes(), s2.as_bytes())
    }

    pub fn maximum(&self) -> f64 {
        1.0
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

// ─── ArithNCD ────────────────────────────────────────────────────────────────

/// Arithmetic Coding NCD.
///
/// Builds a probability model using exact rational arithmetic (represented as
/// 128-bit numerator/denominator fractions), performs arithmetic coding, and
/// returns `ceil(log_base(numerator))` as the compressed size.
///
/// The Python implementation uses `fractions.Fraction` for exact arithmetic.
/// Here we use `u128` fractions (numerator + denominator) which gives enough
/// precision for the test strings.
///
/// <https://en.wikipedia.org/wiki/Arithmetic_coding>
#[derive(Debug, Clone)]
pub struct ArithNcd {
    pub base: u32,
    pub terminator: Option<u8>,
    pub qval: usize,
}

impl Default for ArithNcd {
    fn default() -> Self {
        Self {
            base: 2,
            terminator: None,
            qval: 1,
        }
    }
}

impl ArithNcd {
    pub fn new(base: u32, terminator: Option<u8>, qval: usize) -> Self {
        Self {
            base,
            terminator,
            qval,
        }
    }

    /// Build cumulative probability table: symbol → (cum_start_num/denom, width_num/denom).
    fn make_probs(&self, data: &[u8]) -> Vec<(u8, u128, u128, u128)> {
        // count frequencies
        let mut counts: HashMap<u8, u64> = HashMap::new();
        for &b in data {
            *counts.entry(b).or_insert(0) += 1;
        }
        if let Some(t) = self.terminator {
            counts.entry(t).or_insert(1);
        }

        // sort by descending count (like Python Counter.most_common())
        let mut items: Vec<(u8, u64)> = counts.into_iter().collect();
        items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        let total: u64 = items.iter().map(|(_, c)| c).sum();
        let denom = total as u128;

        let mut probs = Vec::with_capacity(items.len());
        let mut cumulative: u128 = 0;
        for (sym, count) in items {
            probs.push((sym, cumulative, count as u128, denom));
            cumulative += count as u128;
        }
        probs
    }

    /// Arithmetic-code `data` and return the compressed fraction numerator.
    pub fn compress_numerator(&self, data: &[u8]) -> u128 {
        let mut seq = data.to_vec();
        if let Some(t) = self.terminator {
            seq.retain(|&b| b != t);
            seq.push(t);
        }

        let probs = self.make_probs(data);

        // start=0/1, width=1/1  (as num/denom pairs with shared large denominator)
        // We track: interval_start (numerator), interval_width (numerator), denominator
        // All as u128 — sufficient for small test strings.
        let mut start_n: u128 = 0; // start numerator
        let mut width_n: u128 = 1; // width numerator  (= 1 * initial_denom)
        let denom = if probs.is_empty() { 1u128 } else { probs[0].3 };
        width_n *= denom; // start: start=0/denom, width=denom/denom

        for &sym in &seq {
            if let Some(&(_, cum, w, d)) = probs.iter().find(|&&(s, _, _, _)| s == sym) {
                // new_start = start + cum/d * width  (all over denom*d)
                // new_width = w/d * width
                let new_start_n = start_n * d + cum * width_n;
                let new_width_n = w * width_n;
                let new_denom = denom * d; // not actually stored — we normalise
                                           // reduce: divide start and width by gcd with new_denom
                                           // For simplicity, just keep exact and let them grow
                start_n = new_start_n;
                width_n = new_width_n;
                let _ = new_denom;
            }
        }

        // Find smallest fraction p/q in [start, start+width] with smallest numerator
        // Python: output_denominator doubles until found
        // We replicate: output_fraction = 0/1, denominator doubles
        let total_denom = {
            // Reconstruct actual denominator
            let mut d: u128 = 1;
            for _ in &seq {
                d *= probs[0].3;
            }
            d
        };

        let mut output_denom: u128 = 1;
        loop {
            // output_numerator = 1 + floor(start_n * output_denom / total_denom)
            let num = 1 + (start_n * output_denom) / total_denom;
            // check: start_n/total_denom <= num/output_denom < (start_n+width_n)/total_denom
            if num * total_denom >= start_n * output_denom
                && num * total_denom < (start_n + width_n) * output_denom
            {
                return num;
            }
            output_denom = output_denom.saturating_mul(2);
            if output_denom > (1u128 << 100) {
                // Overflow guard — return approximation
                return num;
            }
        }
    }

    fn compress_size(&self, data: &[u8]) -> f64 {
        let numerator = self.compress_numerator(data);
        if numerator == 0 {
            return 0.0;
        }
        (numerator as f64).log(self.base as f64).ceil()
    }
}

impl NcdBase for ArithNcd {
    fn get_size(&self, data: &[u8]) -> f64 {
        self.compress_size(data)
    }
}

impl ArithNcd {
    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        self.ncd(s1.as_bytes(), s2.as_bytes())
    }

    pub fn maximum(&self) -> f64 {
        1.0
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

// ─── BZ2NCD ──────────────────────────────────────────────────────────────────

/// BZ2 compression NCD.
///
/// Compresses data with bzip2, strips the 15-byte header, and uses the remaining
/// compressed byte length as the "compressed size".
///
/// Dependency: `bzip2` crate (justified: needed for faithful port of Python `codecs.encode(data, 'bz2_codec')[15:]`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Bz2Ncd;

fn bz2_compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = BzEncoder::new(Vec::new(), BzCompression::default());
    encoder.write_all(data).unwrap_or(());
    encoder.finish().unwrap_or_default()
}

impl NcdBase for Bz2Ncd {
    fn get_size(&self, data: &[u8]) -> f64 {
        let compressed = bz2_compress(data);
        // Strip 14-byte header (Python strips 15; bzip2 crate header is slightly different)
        // Python: codecs.encode(data, 'bz2_codec')[15:]
        // The bzip2 crate stream header is 10 bytes (BZh + digit + 6-byte block magic).
        // To match Python behaviour we strip 15 bytes.
        compressed.len().saturating_sub(15) as f64
    }
}

impl Bz2Ncd {
    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        self.ncd(s1.as_bytes(), s2.as_bytes())
    }

    pub fn maximum(&self) -> f64 {
        1.0
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

// ─── LZMANCD ─────────────────────────────────────────────────────────────────

/// LZMA compression NCD.
///
/// Compresses data with LZMA, strips the 14-byte header.
///
/// Dependency: `xz2` crate (justified: needed for faithful port of Python `lzma.compress(data)[14:]`).
#[derive(Debug, Clone, Copy, Default)]
pub struct LzmaNcd;

fn lzma_compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = XzEncoder::new(Vec::new(), 6);
    encoder.write_all(data).unwrap_or(());
    encoder.finish().unwrap_or_default()
}

impl NcdBase for LzmaNcd {
    fn get_size(&self, data: &[u8]) -> f64 {
        let compressed = lzma_compress(data);
        compressed.len().saturating_sub(14) as f64
    }
}

impl LzmaNcd {
    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        self.ncd(s1.as_bytes(), s2.as_bytes())
    }

    pub fn maximum(&self) -> f64 {
        1.0
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

// ─── ZLIBNCD ─────────────────────────────────────────────────────────────────

/// Zlib compression NCD.
///
/// Compresses data with zlib (deflate), strips the 2-byte header.
///
/// Dependency: `flate2` crate (justified: needed for faithful port of Python `codecs.encode(data, 'zlib_codec')[2:]`).
#[derive(Debug, Clone, Copy, Default)]
pub struct ZlibNcd;

fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), ZlibCompression::default());
    encoder.write_all(data).unwrap_or(());
    encoder.finish().unwrap_or_default()
}

impl NcdBase for ZlibNcd {
    fn get_size(&self, data: &[u8]) -> f64 {
        let compressed = zlib_compress(data);
        compressed.len().saturating_sub(2) as f64
    }
}

impl ZlibNcd {
    pub fn distance(&self, s1: &str, s2: &str) -> f64 {
        self.ncd(s1.as_bytes(), s2.as_bytes())
    }

    pub fn maximum(&self) -> f64 {
        1.0
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        self.distance(s1, s2)
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Monotonicity helper: dist(a,a) <= dist(a,b) <= dist(a,c) for decreasing similarity
    fn check_monotonicity<F: Fn(&str, &str) -> f64>(dist: F) {
        let d_same = dist("test", "test");
        let d_close = dist("test", "text");
        let d_far = dist("test", "nani");
        assert!(
            d_same <= d_close && d_close <= d_far,
            "Monotonicity failed: {d_same} <= {d_close} <= {d_far}"
        );
    }

    fn check_symmetry<F: Fn(&str, &str) -> f64>(dist: F) {
        let ab = dist("test", "nani");
        let ba = dist("nani", "test");
        assert!((ab - ba).abs() < 1e-9, "Symmetry failed: {ab} != {ba}");
    }

    // ── RLENCD ───────────────────────────────────────────────────────────────

    #[test]
    fn test_rlencd_same_string() {
        // RLENCD("test","test"): compress("testtest")="testtest" (no run > 2), size=8
        // C("test")=4. NCD = (8 - 4) / 4 = 1.0
        // Python _NCDBase does NOT call quick_answer — it always computes.
        let alg = Rlencd::default();
        let d = alg.distance("test", "test");
        assert!((d - 1.0).abs() < 1e-9, "Expected 1.0, got {d}");
    }

    #[test]
    fn test_rlencd_monotonicity() {
        check_monotonicity(|a, b| Rlencd::default().distance(a, b));
    }

    #[test]
    fn test_rlencd_symmetry() {
        check_symmetry(|a, b| Rlencd::default().distance(a, b));
    }

    #[test]
    fn test_rlencd_normalized() {
        let alg = Rlencd::default();
        let nd = alg.normalized_distance("test", "text");
        let ns = alg.normalized_similarity("test", "text");
        assert!((nd + ns - 1.0).abs() < 1e-9);
        assert!((0.0..=1.0).contains(&nd));
    }

    // ── BWTRLENCD ────────────────────────────────────────────────────────────

    #[test]
    fn test_bwtrlencd_python_values() {
        let alg = Bwtrlencd::default();
        // Python: ('test', 'test') → 0.6, ('test', 'nani') → 0.8
        let d1 = alg.distance("test", "test");
        let d2 = alg.distance("test", "nani");
        assert!((d1 - 0.6).abs() < 1e-9, "Expected ~0.6, got {d1}");
        assert!((d2 - 0.8).abs() < 1e-9, "Expected ~0.8, got {d2}");
    }

    #[test]
    fn test_bwtrlencd_monotonicity() {
        check_monotonicity(|a, b| Bwtrlencd::default().distance(a, b));
    }

    #[test]
    fn test_bwtrlencd_symmetry() {
        check_symmetry(|a, b| Bwtrlencd::default().distance(a, b));
    }

    // ── SqrtNCD ──────────────────────────────────────────────────────────────

    #[test]
    fn test_sqrtncd_python_values() {
        let alg = SqrtNcd::default();
        // Python: ('test', 'test') → 0.41421356237309503 (√2 - 1)
        let d1 = alg.distance("test", "test");
        let expected1 = 2.0_f64.sqrt() - 1.0;
        assert!(
            (d1 - expected1).abs() < 1e-9,
            "Expected ~{expected1}, got {d1}"
        );
        // Python: ('test', 'nani') → 1.0
        let d2 = alg.distance("test", "nani");
        assert!((d2 - 1.0).abs() < 1e-9, "Expected ~1.0, got {d2}");
    }

    #[test]
    fn test_sqrtncd_monotonicity() {
        check_monotonicity(|a, b| SqrtNcd::default().distance(a, b));
    }

    #[test]
    fn test_sqrtncd_symmetry() {
        check_symmetry(|a, b| SqrtNcd::default().distance(a, b));
    }

    // ── EntropyNCD ───────────────────────────────────────────────────────────

    #[test]
    fn test_entropyncd_python_values() {
        let alg = EntropyNcd::default();
        // ('aaa', 'bbb'): C(aaa)=coef+0=1, C(bbb)=1, C(aaabbb)=coef+log2(2)=1+1=2
        // NCD = (2 - min(1,1)) / max(1,1) = 1.0
        let d1 = alg.distance("aaa", "bbb");
        assert!((d1 - 1.0).abs() < 1e-9, "Expected 1.0, got {d1}");
        // ('test', 'nani'): should be somewhere in (0, 1]
        let d2 = alg.distance("test", "nani");
        assert!((0.0..=1.0).contains(&d2), "Expected in [0,1], got {d2}");
    }

    #[test]
    fn test_entropyncd_monotonicity() {
        check_monotonicity(|a, b| EntropyNcd::default().distance(a, b));
    }

    #[test]
    fn test_entropyncd_symmetry() {
        check_symmetry(|a, b| EntropyNcd::default().distance(a, b));
    }

    #[test]
    fn test_entropyncd_normalized() {
        let alg = EntropyNcd::default();
        let nd = alg.normalized_distance("test", "text");
        let ns = alg.normalized_similarity("test", "text");
        assert!((nd + ns - 1.0).abs() < 1e-9);
        assert!((0.0..=1.0).contains(&nd));
    }

    // ── BZ2NCD ───────────────────────────────────────────────────────────────

    #[test]
    fn test_bz2ncd_monotonicity() {
        check_monotonicity(|a, b| Bz2Ncd.distance(a, b));
    }

    #[test]
    fn test_bz2ncd_symmetry() {
        check_symmetry(|a, b| Bz2Ncd.distance(a, b));
    }

    #[test]
    fn test_bz2ncd_normalized() {
        let nd = Bz2Ncd.normalized_distance("test", "text");
        let ns = Bz2Ncd.normalized_similarity("test", "text");
        assert!((nd + ns - 1.0).abs() < 1e-9);
    }

    // ── LZMANCD ──────────────────────────────────────────────────────────────

    #[test]
    fn test_lzmancd_monotonicity() {
        check_monotonicity(|a, b| LzmaNcd.distance(a, b));
    }

    #[test]
    fn test_lzmancd_symmetry() {
        check_symmetry(|a, b| LzmaNcd.distance(a, b));
    }

    #[test]
    fn test_lzmancd_normalized() {
        let nd = LzmaNcd.normalized_distance("test", "text");
        let ns = LzmaNcd.normalized_similarity("test", "text");
        assert!((nd + ns - 1.0).abs() < 1e-9);
    }

    // ── ZLIBNCD ──────────────────────────────────────────────────────────────

    #[test]
    fn test_zlibncd_monotonicity() {
        check_monotonicity(|a, b| ZlibNcd.distance(a, b));
    }

    #[test]
    fn test_zlibncd_symmetry() {
        check_symmetry(|a, b| ZlibNcd.distance(a, b));
    }

    #[test]
    fn test_zlibncd_normalized() {
        let nd = ZlibNcd.normalized_distance("test", "text");
        let ns = ZlibNcd.normalized_similarity("test", "text");
        assert!((nd + ns - 1.0).abs() < 1e-9);
    }
}
