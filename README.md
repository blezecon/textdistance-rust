# textdistance-rust

> A faithful, pure-Rust port of the Python [TextDistance](https://github.com/life4/textdistance) library — 35 string-similarity and distance algorithms across five algorithm families, built as a two-person hackathon project.

---

## Table of Contents

- [Why This Exists](#why-this-exists)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Available Algorithms](#available-algorithms)
- [Project Structure](#project-structure)
- [Public API](#public-api)
- [Python Compatibility](#python-compatibility)
- [Testing](#testing)
- [Development](#development)
- [Contributing](#contributing)
- [Roadmap](#roadmap)
- [Credits](#credits)

---

## Why This Exists

[TextDistance](https://github.com/life4/textdistance) is a popular Python library that collects dozens of string-similarity algorithms under a single, consistent interface. It is widely used for fuzzy matching, record linkage, spell checking, and NLP preprocessing.

This repository is an **independent Rust port** of TextDistance, built during a hackathon. The goal is to faithfully translate the original Python implementation into idiomatic, safe Rust while preserving:

- Behavioral compatibility with the Python library
- The same algorithm names and parameters wherever practical
- The same edge-case handling (empty inputs, identical inputs, Unicode)
- The same normalization semantics (`normalized_distance + normalized_similarity == 1.0`)

The project was built by two developers working in parallel using AI-assisted coding. [AGENT.md](./AGENT.md) documents the full development protocol.

---

## Features

- **Pure Rust** — no Python runtime, no FFI, no C extensions required for the core algorithms
- **Safe Rust** — `unsafe` code is explicitly prohibited by the project's coding guidelines
- **35 algorithms** across five families: edit-based, sequence-based, token-based, compression-based, and phonetic
- **Python-compatible behavior** — test values are sourced directly from the Python library's own test suite
- **Comprehensive unit tests** — every algorithm has tests for empty input, identical input, ASCII, Unicode, and different-length cases
- **Idiomatic API** — structs with `Default`, `new()`, and `Base<T>` trait implementations
- **Generic over element type** — most algorithms work over `&[T]` for any `T: PartialEq`, not just `&str`
- **q-gram tokenization** — configurable `qval` parameter on every algorithm, matching Python's behavior
- **Minimal dependencies** — only three external crates, all justified by compression algorithm requirements (`bzip2`, `flate2`, `xz2`)

---

## Installation

This crate is not yet published to [crates.io](https://crates.io). To use it from GitHub, add the following to your `Cargo.toml`:

```toml
[dependencies]
textdistance-rust = { git = "https://github.com/blezecon/textdistance-rust" }
```

Once a release is published to crates.io, installation will be:

```toml
[dependencies]
textdistance-rust = "0.1"
```

> **Note:** The crate requires **Rust stable** toolchain. It was developed with Rust edition 2024 (`edition = "2024"` in `Cargo.toml`).

---

## Quick Start

All algorithms are struct types that implement the `Base<T>` trait. Construct a struct (using `::default()` for standard parameters or `::new(...)` for custom ones), then call methods on it.

All methods accept `&[T]` slices. For string comparison, collect the string into a `Vec<char>` first.

```rust
use textdistance_rust::edit_based::{Levenshtein, Hamming, JaroWinkler};
use textdistance_rust::token_based::{Jaccard, Cosine};
use textdistance_rust::base::Base;

fn main() {
    // Helper: convert &str to Vec<char>
    let s1: Vec<char> = "kitten".chars().collect();
    let s2: Vec<char> = "sitting".chars().collect();

    // ── Levenshtein ─────────────────────────────────────────────────────────
    let lev = Levenshtein::default();
    println!("Levenshtein distance:             {}", lev.distance(&s1, &s2));
    // → 3.0
    println!("Levenshtein normalized distance:  {}", lev.normalized_distance(&s1, &s2));
    // → 0.4285...
    println!("Levenshtein normalized similarity:{}", lev.normalized_similarity(&s1, &s2));
    // → 0.5714...

    // ── Hamming ─────────────────────────────────────────────────────────────
    let t1: Vec<char> = "test".chars().collect();
    let t2: Vec<char> = "text".chars().collect();
    let ham = Hamming::default();
    println!("Hamming distance: {}", ham.distance(&t1, &t2));
    // → 1.0

    // ── Jaro-Winkler ────────────────────────────────────────────────────────
    let jw = JaroWinkler::default();  // winklerize = true by default
    println!("Jaro-Winkler similarity: {}", jw.similarity(&t1, &t2));
    // → ~0.9333...

    // ── Jaccard ─────────────────────────────────────────────────────────────
    let n1: Vec<char> = "nelson".chars().collect();
    let n2: Vec<char> = "neilsen".chars().collect();
    let jac = Jaccard::default();
    println!("Jaccard similarity: {}", jac.similarity(&n1, &n2));
    // → 0.625 (5 / 8)

    // ── Cosine ──────────────────────────────────────────────────────────────
    let cos = Cosine::default();
    println!("Cosine similarity: {}", cos.similarity(&n1, &n2));
    // → ~0.7715 (5 / sqrt(6 * 7))
}
```

### Compression-based and token-based algorithms with string APIs

The compression-based algorithms and `MongeElkan` expose a string-level API directly (accepting `&str`), since their semantics are inherently string-oriented:

```rust
use textdistance_rust::compression_based::{Rlencd, Bz2Ncd, EntropyNcd};
use textdistance_rust::token_based::MongeElkan;

fn main() {
    // ── RLENCD (Run-Length Encoding NCD) ────────────────────────────────────
    let rle = Rlencd::default();
    println!("RLENCD distance: {}", rle.distance("test", "text"));

    // ── BZ2 NCD ─────────────────────────────────────────────────────────────
    let bz2 = Bz2Ncd;
    println!("BZ2NCD distance: {}", bz2.distance("test", "nani"));

    // ── Entropy NCD ─────────────────────────────────────────────────────────
    let ent = EntropyNcd::default();
    println!("EntropyNCD normalized similarity: {}", ent.normalized_similarity("test", "test"));
    // → 0.0 (identical strings give distance = 1.0 for entropy-based NCD)

    // ── MongeElkan ──────────────────────────────────────────────────────────
    let me = MongeElkan::default();
    println!("MongeElkan similarity: {}", me.similarity("test", "text"));
}
```

---

## Available Algorithms

**Total implemented: 35 algorithms** across 5 categories.

### Edit-Based (`src/edit_based.rs`)

| Algorithm | Struct | Type | Notes |
|---|---|---|---|
| Hamming | `Hamming<T>` | Distance | Positional mismatch count; `truncate` and `test_func` options |
| Levenshtein | `Levenshtein<T>` | Distance | Two-row DP; custom `test_func` |
| Damerau-Levenshtein | `DamerauLevenshtein<T>` | Distance | `restricted` (OSA) and `unrestricted` modes |
| Jaro | `Jaro` | Similarity | Window-based character matching |
| Jaro-Winkler | `JaroWinkler` | Similarity | Jaro + optional prefix bonus (`winklerize`) and long-string boost |
| StrCmp95 | `StrCmp95` | Similarity | Jaro-Winkler variant with 36-pair phonetic/OCR credit matrix |
| Needleman-Wunsch | `NeedlemanWunsch<T>` | Alignment | Global alignment; custom `gap_cost` and `sim_func` |
| Gotoh | `Gotoh<T>` | Alignment | Affine-gap global alignment; `gap_open` and `gap_ext` |
| Smith-Waterman | `SmithWaterman<T>` | Alignment | Local alignment; custom `gap_cost` and `sim_func` |
| MLIPNS | `MLIPNS` | Similarity | Iterative Hamming relaxation; `threshold` and `maxmismatches` |

### Sequence-Based (`src/sequence_based.rs`)

| Algorithm | Struct | Type | Notes |
|---|---|---|---|
| Longest Common Subsequence | `LCSSeq<T>` | Similarity | DP for 2 inputs; recursive for N inputs |
| Longest Common Substring | `LCSStr` | Similarity | difflib-style for short inputs; n-gram scan for long/N inputs |
| Ratcliff-Obershelp | `RatcliffObershelp` | Similarity | Gestalt pattern matching; recursive LCS-based |

### Phonetic (`src/phonetic.rs`)

| Algorithm | Struct | Type | Notes |
|---|---|---|---|
| MRA | `MRA` | Similarity | Match Rating Approach; vowel stripping, dedup, truncation |
| Editex | `Editex` | Distance | Letter-class edit distance; 10 phonetic groups; `local` mode |

### Token-Based (`src/token_based.rs`)

| Algorithm | Struct | Type | Notes |
|---|---|---|---|
| Jaccard | `Jaccard` | Similarity | `\|A ∩ B\| / \|A ∪ B\|`; `qval` and `as_set` |
| Sørensen-Dice | `Sorensen` | Similarity | `2\|A ∩ B\| / (\|A\| + \|B\|)`; `qval` and `as_set` |
| Tversky | `Tversky` | Similarity | Generalization of Jaccard/Sørensen; `ks`, `bias`, `as_set` |
| Overlap | `Overlap` | Similarity | `\|A ∩ B\| / min(\|A\|, \|B\|)`; `qval` and `as_set` |
| Cosine (Ochiai) | `Cosine` | Similarity | `\|A ∩ B\| / sqrt(\|A\| · \|B\|)`; `qval` and `as_set` |
| Tanimoto | `Tanimoto` | Similarity | `log₂(Jaccard)`; returns `-∞` when Jaccard is 0 |
| Bag | `Bag` | Distance | `max(\|A ∖ (A ∩ B)\|, \|B ∖ (A ∩ B)\|)` |
| Monge-Elkan | `MongeElkan` | Similarity | Token-level max-similarity; uses `DamerauLevenshtein` internally; `symmetric` mode |

### Compression-Based (`src/compression_based.rs`)

All compression-based algorithms use the **Normalized Compression Distance (NCD)** formula:

```
NCD(x, y) = (C(xy) - min(C(x), C(y))) / max(C(x), C(y))
```

where `C(·)` is the compressed size supplied by each variant. Both orderings of concatenation (`xy` and `yx`) are tried and the minimum is used.

| Algorithm | Struct | Compressor | External Crate? |
|---|---|---|---|
| RLENCD | `Rlencd` | Run-length encoding (pure Rust) | No |
| BWTRLENCD | `Bwtrlencd` | Burrows-Wheeler Transform + RLE (pure Rust) | No |
| SqrtNCD | `SqrtNcd` | `Σ √freq(char)` (pure Rust) | No |
| EntropyNCD | `EntropyNcd` | Shannon entropy (pure Rust) | No |
| ArithNCD | `ArithNcd` | Arithmetic coding with `u128` fractions (pure Rust) | No |
| BZ2NCD | `Bz2Ncd` | bzip2 compression | `bzip2 = "0.5"` |
| LZMANCD | `LzmaNcd` | LZMA compression | `xz2 = "0.1"` |
| ZLIBNCD | `ZlibNcd` | zlib (deflate) compression | `flate2 = "1.0"` |

### Simple (`src/simple.rs`)

| Algorithm | Struct | Type | Notes |
|---|---|---|---|
| Prefix | `Prefix<T>` | Similarity | Common prefix length; `qval` and `sim_test` |
| Postfix | `Postfix<T>` | Similarity | Common postfix length; delegates to `Prefix` after reversing |
| Length | `Length` | Distance | Absolute length difference |
| Identity | `Identity` | Similarity | 1 if identical, 0 otherwise |
| Matrix | `Matrix<T>` | Similarity | Lookup table with match/mismatch costs; symmetric option |

---

## Project Structure

```
textdistance-rust/
├── Cargo.toml              # Package metadata and dependencies
├── Cargo.lock              # Locked dependency versions
├── LICENSE                 # MIT License
├── README.md               # This file
├── AGENT.md                # AI/developer workflow protocol
├── PERSON1.md              # Developer 1 assignment (infrastructure + edit/sequence/phonetic/simple)
├── PERSON2.md              # Developer 2 assignment (token-based + compression-based)
└── src/
    ├── lib.rs              # Public module declarations (frozen after bootstrap)
    ├── base.rs             # Core Base<T> trait + counter helpers (frozen after bootstrap)
    ├── types.rs            # SimFunc<T> and TestFunc<T> type aliases (frozen after bootstrap)
    ├── utils.rs            # find_ngrams<T> utility (frozen after bootstrap)
    ├── simple.rs           # Prefix, Postfix, Length, Identity, Matrix
    ├── edit_based.rs       # Hamming, Levenshtein, DamerauLevenshtein, Jaro, JaroWinkler,
    │                       # StrCmp95, NeedlemanWunsch, Gotoh, SmithWaterman, MLIPNS
    ├── sequence_based.rs   # LCSSeq, LCSStr, RatcliffObershelp
    ├── phonetic.rs         # MRA, Editex
    ├── token_based.rs      # Jaccard, Sorensen, Tversky, Overlap, Cosine, Tanimoto, Bag,
    │                       # MongeElkan
    └── compression_based.rs # Rlencd, Bwtrlencd, SqrtNcd, EntropyNcd, ArithNcd,
                             # Bz2Ncd, LzmaNcd, ZlibNcd
```

**Infrastructure files** (`lib.rs`, `base.rs`, `types.rs`, `utils.rs`) are frozen after the initial bootstrap commit and are not modified by either developer.

---

## Public API

### The `Base<T>` Trait

Every algorithm (except `MongeElkan` and the compression-based structs) implements `Base<T>`:

```rust
pub trait Base<T: PartialEq> {
    fn distance(&self, s1: &[T], s2: &[T]) -> f64;
    fn similarity(&self, s1: &[T], s2: &[T]) -> f64;
    fn maximum(&self, s1: &[T], s2: &[T]) -> f64;
    fn normalized_distance(&self, s1: &[T], s2: &[T]) -> f64;   // always in [0.0, 1.0]
    fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> f64; // always in [0.0, 1.0]
}
```

Default implementations mirror the Python base class:
- `similarity = maximum - distance` (for distance-first algorithms)
- `distance = maximum - similarity` (for similarity-first algorithms)
- `normalized_distance = distance / maximum` (returns `0.0` when `maximum == 0.0`)
- `normalized_similarity = 1.0 - normalized_distance`

The invariant `normalized_distance + normalized_similarity == 1.0` holds for all algorithms.

> **Exception — alignment algorithms:** `NeedlemanWunsch` and `Gotoh` override `normalized_distance` and `normalized_similarity` with custom formulas that rescale between `minimum` (most negative score) and `maximum`, matching Python's implementation.

> **Exception — `Tanimoto`:** Returns `log₂(Jaccard)`, which is ≤ 0; standard normalization does not apply.

### Helper Functions (`src/base.rs`)

```rust
pub fn is_ident<T: PartialEq>(s1: &[T], s2: &[T]) -> bool;
pub fn quick_answer_distance<T: PartialEq>(s1: &[T], s2: &[T], max_val: f64) -> Option<f64>;
pub fn quick_answer_similarity<T: PartialEq>(s1: &[T], s2: &[T], max_val: f64) -> Option<f64>;
pub fn get_counter<T: Hash + Eq + Clone>(seq: &[T]) -> HashMap<T, usize>;
pub fn intersect_counters<T: Hash + Eq + Clone>(c1: &HashMap<T, usize>, c2: &HashMap<T, usize>) -> HashMap<T, usize>;
pub fn union_counters<T: Hash + Eq + Clone>(c1: &HashMap<T, usize>, c2: &HashMap<T, usize>) -> HashMap<T, usize>;
pub fn sum_counters<T: Hash + Eq + Clone>(c1: &HashMap<T, usize>, c2: &HashMap<T, usize>) -> HashMap<T, usize>;
pub fn count_counters<T: Hash + Eq>(counter: &HashMap<T, usize>, as_set: bool) -> usize;
```

### Utility (`src/utils.rs`)

```rust
/// Extracts n-grams from a slice of elements.
pub fn find_ngrams<T: Clone>(input_list: &[T], n: usize) -> Vec<Vec<T>>;
```

### Type Aliases (`src/types.rs`)

```rust
pub type SimFunc<T>  = Option<fn(&T, &T) -> f64>;  // element similarity function
pub type TestFunc<T> = Option<fn(&T, &T) -> bool>; // element equality test
```

---

## Python Compatibility

The Python [TextDistance](https://github.com/life4/textdistance) library is treated as the **canonical specification** for this project. Every implementation decision is made with the following priorities (from `AGENT.md`):

1. Correctness
2. Python compatibility
3. Complete algorithm coverage
4. Idiomatic Rust
5. Performance
6. Minimal dependencies

### What is ported

- Algorithm names (e.g., `DamerauLevenshtein`, `JaroWinkler`, `RatcliffObershelp`)
- Default parameter values (e.g., `qval=1`, `truncate=false`, `restricted=true`)
- Normalization formulas
- Edge-case handling (empty inputs, identical inputs)
- The `qval` q-gram tokenization parameter
- The `as_set` parameter for token-based algorithms
- Counter operations (`get_counter`, `intersect_counters`, `union_counters`, `sum_counters`)

### What is not ported

- The `external` parameter (Python-specific, delegates to C libraries)
- `vector_based.py` algorithms (marked incomplete/draft in Python)
- Python `qval=0` word-splitting behavior (not meaningful for generic `&[T]` API)

### Known behavioral deviations

All intentional deviations are documented in the source code with `Deviation:` comments:

| Algorithm | Deviation |
|---|---|
| `Prefix`/`Postfix` | `qval > 1` computes q-gram postfix length instead of crashing (Python raises `TypeError`) |
| `LCSSeq` | `qval > 1` computes LCS over q-gram tokens instead of crashing (Python raises `TypeError`) |
| `Gotoh` | Returns a finite score for empty inputs instead of raising `IndexError` |
| `MRA`/`Editex` | Uppercasing uses `char::to_uppercase`'s first code point (ß → S, not SS) |

### Differential testing

Test values for every algorithm are sourced directly from the Python library's own test suite (files referenced in `PERSON1.md` and `PERSON2.md`). The Rust tests assert exact or near-exact (`< 1e-6`) matches against those values.

---

## Testing

### Run all tests

```bash
cargo test
```

### Run tests for a specific module

```bash
cargo test --test-thread 1 -- edit_based
cargo test --test-thread 1 -- token_based
cargo test --test-thread 1 -- compression_based
cargo test --test-thread 1 -- sequence_based
cargo test --test-thread 1 -- phonetic
cargo test --test-thread 1 -- simple
cargo test --test-thread 1 -- utils
```

### Test coverage

Every algorithm has tests covering:

| Test scenario | Covered |
|---|---|
| Empty input (`""`, `""`) | ✅ |
| One empty input (`""`, `"abc"`) | ✅ |
| Identical non-empty inputs | ✅ |
| ASCII strings | ✅ |
| Unicode / multibyte characters (e.g., `café`, `héllo`) | ✅ |
| Different-length inputs | ✅ |
| Python library expected values | ✅ |
| Normalization invariant (`nd + ns == 1.0`) | ✅ |
| Algorithm-specific invariants (e.g., Tversky ↔ Jaccard, anagram bag distance = 0) | ✅ |

---

## Development

### Prerequisites

- Rust stable toolchain (`rustup install stable`)
- System libraries for compression crates (bzip2, xz/lzma dev headers)

### Build

```bash
cargo build
```

### Format

```bash
cargo fmt
```

### Lint

```bash
cargo clippy
```

### Test

```bash
cargo test
```

### Check (no codegen, fast)

```bash
cargo check
```

### Full verification pass (required before committing)

```bash
cargo fmt && cargo clippy && cargo test
```

> The project has **no benchmarks directory** and **no comparison scripts** at this time. Benchmark and comparison infrastructure has not been created.

---

## Contributing

1. **Read `AGENT.md` first.** It defines the development protocol, Definition of Done, and code style rules.
2. **Read both `PERSON1.md` and `PERSON2.md`** to understand file ownership boundaries.
3. **One algorithm per PR.** Each pull request should implement exactly one algorithm with its tests.
4. **Always run the full verification pass:**
   ```bash
   cargo fmt && cargo clippy && cargo test
   ```
5. **Preserve Python compatibility.** If you must deviate, document the deviation with a `Deviation:` comment in the source.
6. **Follow the Definition of Done** (from `AGENT.md`):
   - Implementation compiles
   - Behavior matches Python
   - Public API is documented with `///` doc comments
   - Unit tests cover all required cases
   - All existing tests continue to pass
   - `cargo fmt`, `cargo clippy`, and `cargo test` pass
   - No unrelated files were modified

### Code style

- Safe Rust only — `unsafe` is prohibited
- Prefer iterators, slices, traits, generics, and borrowing
- Avoid unnecessary allocations and cloning
- Avoid `unwrap()` in library code
- Prefer readability over optimization
- Prefer `&[T]` over `&Vec<T>`

---

## Roadmap

The following items are **obviously incomplete** based on the repository state:

- [ ] **Publish to crates.io** — the crate is not yet published (version `0.1.0`, no publish metadata beyond `Cargo.toml`)
- [ ] **`MongeElkan` checklist** — the `PERSON2.md` checklist shows MongeElkan as complete (`[x]`), but `MongeElkan` does **not** implement the `Base<T>` trait; it exposes string-level methods directly (`similarity(&str, &str)`, `distance(&str, &str)`). Full trait integration is not done.
- [ ] **Benchmarks** — no `benches/` directory exists; performance characteristics are not measured
- [ ] **Comparison scripts** — no scripts exist to compare Rust output against Python output programmatically
- [ ] **`examples/` directory** — no standalone examples exist; only inline doc tests
- [ ] **`vector_based` algorithms** — explicitly out of scope (marked draft/incomplete in Python); not ported
- [ ] **`qval=0` word-splitting** — Python's `qval=0` splits strings by whitespace into word tokens; this is not implemented for the generic `&[T]` API

---

## Credits

### Original Python project

**TextDistance** — [https://github.com/life4/textdistance](https://github.com/life4/textdistance)

The Python TextDistance library is the canonical specification for this port. All algorithm logic, default parameters, edge-case behavior, and test expected values are derived from it.

### This Rust port

Built by **blezecon** and **NoE114** during a hackathon.

- [blezecon](mailto:dfyyasdd@gmail.com) — infrastructure bootstrap, edit-based, sequence-based, phonetic, and simple algorithms
- [NoE114](mailto:rajdey8787@gmail.com) — token-based and compression-based algorithms
