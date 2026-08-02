# PERSON 2 — Assignment: Token-Based + Compression-Based Algorithms

---

## Collaboration Rules

Before implementing any code:

- Read the repository root [`AGENT.md`](file:///home/blezecon/Code/textdistance-rust/AGENT.md).
- Read [`./AGENT.md`](file:///home/blezecon/Code/textdistance-rust/./AGENT.md).
- Read BOTH [`PERSON1.md`](file:///home/blezecon/Code/textdistance-rust/./PERSON1.md) and [`PERSON2.md`](file:///home/blezecon/Code/textdistance-rust/./PERSON2.md).

Understand the ownership boundaries before making changes.

Do not modify files owned by the other developer.

### Freeze Infrastructure

After the bootstrap commit, the following files become READ ONLY:

- `src/lib.rs`
- `src/base.rs`
- `src/types.rs`
- `src/utils.rs`

Neither developer nor their AI assistant should modify these files unless fixing a critical bug.

### Cargo.toml Ownership

Avoid merge conflicts. Person 1 is the sole owner of `Cargo.toml`.

Person 2 MUST NOT modify `Cargo.toml` directly. If Person 2 needs an additional dependency:
- Document the required crate
- Explain why it is needed
- Ask Person 1 to add it

If shared infrastructure changes are required, leave a TODO or open an issue instead of implementing another person's assignment.

For every algorithm:

1. Read the Python implementation first and explain it in 3–5 sentences before writing any Rust code.
2. Implement the Rust version.
3. Verify behavior matches Python.
4. Add unit tests.
5. Only optimize after correctness has been verified.

---

## Architecture Rules

The purpose of this repository is to faithfully port the Python implementation.

Therefore:

- Do NOT redesign the architecture.
- Do NOT replace algorithms with different implementations unless required for Rust.
- Preserve observable behavior.
- Preserve edge cases.
- Preserve public API semantics whenever practical.
- Prefer compatibility over clever Rust abstractions.
- Correctness is more important than optimization.

---

## Git Workflow

- Each developer works on their own branch.
- Commit frequently.
- Prefer one algorithm per commit.
- Run:
  - `cargo fmt`
  - `cargo clippy`
  - `cargo test`
  before committing.
- Rebase onto the latest main before opening a pull request.
- Never force-push shared branches.

---

## AI Scope Rules

The AI assistant should:

- Only modify files owned by this assignment.
- Never edit another developer's files.
- Never perform large-scale refactors.
- Never rename modules without instruction.
- Never reorganize the project structure.
- Never move files.
- Never change shared interfaces without approval.
- Never modify infrastructure after the bootstrap phase.
- Stop and ask for clarification instead of making architectural decisions.

### One Algorithm at a Time

The AI assistant must implement **exactly one algorithm at a time**.

For each algorithm, the workflow is:

1. Read the corresponding Python implementation.
2. Explain the Python implementation in 3–5 sentences.
3. Implement the Rust version.
4. Add unit tests.
5. Verify behavior matches the Python implementation.
6. Run:
   - `cargo fmt`
   - `cargo clippy`
   - `cargo test`
7. Ensure the algorithm satisfies the project's Definition of Done.
8. Stop and wait for the next instruction.

Do NOT automatically continue with the next algorithm.

Do NOT batch multiple algorithms into a single implementation session unless explicitly instructed.

### Ambiguous or Incomplete Python Code

The Python implementation is the specification.

If the Python implementation is:

- incomplete
- ambiguous
- marked TODO
- intentionally unimplemented
- dependent on Python-specific behavior that has no obvious Rust equivalent

then:

- do NOT invent behavior
- do NOT guess the intended implementation
- explain the ambiguity
- identify the relevant Python source
- stop and ask for clarification before writing Rust code

Compatibility with the Python implementation is more important than creativity.

---

## Definition of Done

An algorithm is considered complete only when all of the following are true:

- The Rust implementation compiles successfully.
- Behavior matches the Python implementation.
- Unit tests have been added.
- Existing tests continue to pass.
- `cargo fmt` passes.
- `cargo clippy` passes without introducing new warnings.
- `cargo test` passes.
- Public APIs are documented where appropriate.
- No files outside the assignment ownership were modified.

Do not consider an algorithm complete until every item above has been satisfied.

---

## Overview

You are responsible for:

1. **Token-based algorithms** — Jaccard, Sorensen, Tversky, Overlap, Cosine, Tanimoto, MongeElkan, Bag
2. **Compression-based algorithms** — ArithNCD, RLENCD, BWTRLENCD, SqrtNCD, EntropyNCD, BZ2NCD, LZMANCD, ZLIBNCD

Person 1 creates the shared infrastructure in a single bootstrap commit.
Once that commit is pushed and you have pulled it, you can work completely independently.
You do not need to wait for Person 1's algorithms to be finished (except MongeElkan — see below).

---

## Dependency Graph

```
Shared Infrastructure (bootstrap commit — Person 1)
├── Cargo.toml
├── src/lib.rs (all module declarations)
├── src/base.rs (traits + counter helpers)
├── src/types.rs (type aliases)
├── src/utils.rs (find_ngrams)
│
├── src/simple.rs ──────────────── Person 1
├── src/edit_based.rs ──────────── Person 1
├── src/sequence_based.rs ──────── Person 1
├── src/phonetic.rs ────────────── Person 1
│
├── src/token_based.rs ─────────── Person 2 (YOU)
│   └── MongeElkan depends on:
│       ├── edit_based::DamerauLevenshtein (Person 1)
│       └── edit_based::JaroWinkler (Person 1, for tests only)
│
└── src/compression_based.rs ───── Person 2 (YOU)
```

Key cross-module dependency: `MongeElkan` (your code in `token_based.rs`) imports `DamerauLevenshtein` from Person 1's `edit_based.rs`. Implement MongeElkan **last**, after Person 1 has pushed `DamerauLevenshtein`.

All other algorithms in your scope have **zero dependencies** on Person 1's algorithm implementations. They only depend on the bootstrap infrastructure (`base.rs`, `types.rs`, `utils.rs`).

---

## Bootstrap Timeline

### Phase 0 — Bootstrap (Person 1 only, you wait)

- Person 1 creates the shared infrastructure in a **single commit**.
- This commit includes: `Cargo.toml`, `src/lib.rs`, `src/base.rs`, `src/types.rs`, `src/utils.rs`, and **empty stub files** for every algorithm module (including your `token_based.rs` and `compression_based.rs`).
- Person 1 commits and pushes.
- **You pull the latest changes.**

> After pulling, `cargo check` should pass. You now have the base traits, counter helpers, and
> utility functions available. You can begin implementing immediately.

### Phase 1 — Parallel Development (both developers)

- Both developers work independently on their own files.
- You implement all token-based algorithms (except MongeElkan) and all compression-based algorithms.
- No coordination needed.

### Phase 2 — Integration

- Both developers run `cargo test` on the merged codebase.
- Fix any integration issues.
- Final `cargo fmt` + `cargo clippy` pass.

---

## File Ownership

Every file has exactly one owner. Shared files are only modified during bootstrap.

| File | Owner | After Bootstrap |
|---|---|---|
| `./Cargo.toml` | Person 1 | **Sole Owner: Person 1** — Person 2 MUST NOT modify `Cargo.toml` directly. If you need a dependency: document crate, explain why needed, ask Person 1 to add it. |
| `./src/lib.rs` | Person 1 (bootstrap) | **READ ONLY / Frozen** — neither developer nor AI assistant modifies after bootstrap unless fixing a critical bug. |
| `./src/base.rs` | Person 1 (bootstrap) | **READ ONLY / Frozen** — neither developer nor AI assistant modifies after bootstrap unless fixing a critical bug. |
| `./src/types.rs` | Person 1 (bootstrap) | **READ ONLY / Frozen** — neither developer nor AI assistant modifies after bootstrap unless fixing a critical bug. |
| `./src/utils.rs` | Person 1 (bootstrap) | **READ ONLY / Frozen** — neither developer nor AI assistant modifies after bootstrap unless fixing a critical bug. |
| `./src/simple.rs` | Person 1 | Do NOT modify. |
| `./src/edit_based.rs` | Person 1 | Do NOT modify. Import `DamerauLevenshtein` and `JaroWinkler` from it. |
| `./src/sequence_based.rs` | Person 1 | Do NOT modify. |
| `./src/phonetic.rs` | Person 1 | Do NOT modify. |
| `./src/token_based.rs` | **You** | You are the sole owner. Populate the empty stub. |
| `./src/compression_based.rs` | **You** | You are the sole owner. Populate the empty stub. |

---

## Scope Boundaries

### You MUST touch

- `./src/token_based.rs` — your algorithm module
- `./src/compression_based.rs` — your algorithm module

### You MUST NOT touch

- `./Cargo.toml` — owned solely by Person 1. Person 2 MUST NOT modify `Cargo.toml` directly. If you need a dependency, document it, explain why needed, and ask Person 1 to add it.
- `./src/lib.rs` — already contains your module declarations
- `./src/base.rs` — read-only (import from it)
- `./src/types.rs` — read-only (import from it)
- `./src/utils.rs` — read-only (import from it)
- `./src/edit_based.rs` — read-only (import from it for MongeElkan)
- `./src/simple.rs` — owned by Person 1
- `./src/sequence_based.rs` — owned by Person 1
- `./src/phonetic.rs` — owned by Person 1
- Any Python files under `textdistance/` — read-only reference
- Any files outside `./`

---

## Reference Python Files

Read these before implementing. They are the specification:

| Rust module | Python reference |
|---|---|
| `token_based.rs` | [`textdistance/algorithms/token_based.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/token_based.py) |
| `compression_based.rs` | [`textdistance/algorithms/compression_based.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/compression_based.py) |

Also read these to understand the infrastructure you import from:

| Dependency | Python reference |
|---|---|
| Base traits | [`textdistance/algorithms/base.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/base.py) |
| Types | [`textdistance/algorithms/types.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/types.py) |
| Utils | [`textdistance/utils.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/utils.py) |

---

## Python Test Files (for expected values)

| Test directory | Algorithms covered |
|---|---|
| [`tests/test_token/`](file:///home/blezecon/Code/textdistance-rust/tests/test_token/) | Bag, Cosine, Jaccard, MongeElkan, Overlap, Sorensen |
| [`tests/test_compression/`](file:///home/blezecon/Code/textdistance-rust/tests/test_compression/) | ArithNCD, BWTRLENCD, BZ2NCD, EntropyNCD, SqrtNCD, common compression tests |
| [`tests/test_common.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_common.py) | Common interface tests (normalization, symmetry, etc.) |

---

## Recommended Implementation Order

### Phase 1: Token-Based Algorithms (no dependency on Person 1's algorithms)

All token-based algorithms (except MongeElkan) depend only on the bootstrap infrastructure.
They all use Counter-based set operations from `base.rs` and follow a similar pattern:

1. Call `quick_answer` (short-circuit for empty/identical)
2. Convert sequences to Counters via `get_counters`
3. Compute intersection / union / sum of counters
4. Return a ratio

Implement simplest to most complex:

- [ ] **1.1** `Jaccard`
  - Jaccard index: `|A ∩ B| / |A ∪ B|`
  - Parameters: `qval=1`, `as_set=false`
  - `maximum` is always 1
  - Similarity-first (Python `BaseSimilarity`)
  - Python: ~35 lines
  - Test values:
    - `('test', 'text')` → `3.0 / 5` = `0.6`
    - `('nelson', 'neilsen')` → `5.0 / 8` = `0.625`
    - `('decide', 'resize')` → `3.0 / 9` ≈ `0.333`
- [ ] **1.2** `Sorensen`
  - Sørensen–Dice coefficient: `2 * |A ∩ B| / (|A| + |B|)`
  - Parameters: `qval=1`, `as_set=false`
  - `maximum` is always 1
  - Python: ~30 lines
  - Test values:
    - `('test', 'text')` → `2.0 * 3 / 8` = `0.75`
- [ ] **1.3** `Overlap`
  - Overlap coefficient: `|A ∩ B| / min(|A|, |B|)`
  - Parameters: `qval=1`, `as_set=false`
  - `maximum` is always 1
  - Python: ~35 lines
  - Test values:
    - `('test', 'text')` → `3.0 / 4` = `0.75`
    - `('testme', 'textthis')` → `4.0 / 6` ≈ `0.667`
    - `('nelson', 'neilsen')` → `5.0 / 6` ≈ `0.833`
- [ ] **1.4** `Cosine`
  - Cosine similarity: `|A ∩ B| / (|A| * |B|)^(1/N)` where N = number of sequences
  - Parameters: `qval=1`, `as_set=false`
  - `maximum` is always 1
  - Uses `reduce` for product of sequence counts
  - Python: ~35 lines
  - Test values:
    - `('test', 'text')` → `3.0 / 4` = `0.75`
    - `('nelson', 'neilsen')` → `5.0 / sqrt(6 * 7)` ≈ `0.7715`
- [ ] **1.5** `Tversky`
  - Parametric Tversky index: `|A ∩ B| / (|A ∩ B| + α|A\B| + β|B\A|)`
  - Parameters: `qval=1`, `ks` (alpha/beta weights, default `[1, 1]`), `bias` (optional), `as_set=false`
  - Has a special two-sequence formula when `bias` is not None
  - `maximum` is always 1
  - Python: ~55 lines
  - **Test**: Jaccard ≡ Tversky(ks=[1,1]) and Sorensen ≡ Tversky(ks=[0.5,0.5])
- [ ] **1.6** `Tanimoto`
  - Wraps `Jaccard`: returns `log₂(jaccard_result)`, or `-infinity` if result is 0
  - `maximum` is always 1
  - Python: ~10 lines
- [ ] **1.7** `Bag`
  - Distance metric (NOT similarity): extends `Base`
  - `max(|A \ (A ∩ B)|, |B \ (A ∩ B)|)` for each sequence minus intersection
  - `maximum` is `max(len(s1), len(s2))` (default)
  - Python: ~5 lines
  - Test values:
    - `('qwe', 'qwe')` → `0`
    - `('qwe', 'erty')` → `3`
    - `('qwe', 'ewq')` → `0` (anagram = same bag)
    - `('qwe', 'rtys')` → `4`

### Phase 2: Compression-Based Algorithms (no dependency on Person 1's algorithms)

These are self-contained. They only need the base trait from `base.rs`.

- [ ] **2.1** NCD base implementation (internal struct/trait for Normalized Compression Distance)
  - NCD formula: `(C(xy) - min(C(x), C(y))) / max(C(x), C(y))`
  - `C(xy)` = compressed size of concatenation (tries all permutations, takes minimum)
  - `maximum` is always 1
  - Each variant provides `_compress(data)` and `_get_size(data) -> f64`
  - Python: ~40 lines
- [ ] **2.2** `RLENCD` — Run-Length Encoding NCD
  - `_compress`: groups consecutive identical elements, encodes as `count + element` if count > 2
  - Pure algorithm, no external dependencies
  - Python: ~15 lines
- [ ] **2.3** `BWTRLENCD` — Burrows-Wheeler Transform + RLE NCD
  - Extends `RLENCD`
  - `_compress`: applies BWT (sort all rotations, take last column) then RLE
  - Uses a `terminator` character (default `'\0'`)
  - Python: ~20 lines
  - Test values:
    - `('test', 'test')` → `0.6`
    - `('test', 'nani')` → `0.8`
- [ ] **2.4** `SqrtNCD` — Square Root NCD
  - `_compress`: for each element, compressed size = `√count`
  - `_get_size`: sum of all `√count` values
  - Python: ~15 lines
  - Test values:
    - `('test', 'test')` → `0.41421356237309503` (√2 - 1)
    - `('test', 'nani')` → `1.0`
- [ ] **2.5** `EntropyNCD` — Entropy-based NCD
  - `_compress`: Shannon entropy = `-Σ p·log₂(p)` where p = count/total
  - `_get_size`: `coef + entropy`
  - Parameters: `qval=1`, `coef=1`, `base=2`
  - Python: ~30 lines
  - Test values:
    - `('test', 'test')` → `1.0`
    - `('aaa', 'bbb')` → `0.0`
    - `('test', 'nani')` → `0.6`
- [ ] **2.6** `ArithNCD` — Arithmetic Coding NCD
  - Builds probability model using exact rational arithmetic
  - `_make_probs`: builds cumulative probability table from Counter
  - `_get_range`: computes arithmetic coding interval
  - `_compress`: finds shortest fraction in the interval
  - `_get_size`: `ceil(log(numerator, base))`
  - Parameters: `base=2`, `terminator=None`, `qval=1`
  - Python: ~70 lines
  - **Dependency note**: Rust has no built-in `Fraction`. Options:
    - Use the `num-rational` crate (add to `Cargo.toml`)
    - Implement a minimal rational number type
    - Ask for clarification if unsure
  - Test values:
    - `('test', 'test')` → `1.0` (similarity)
    - `('test', 'nani')` → `2.1666666666666665` (similarity, not normalized)
    - `_compress('BANANA').numerator` → `1525`
- [ ] **2.7** `BZ2NCD` — BZ2 binary compressor NCD
  - Converts strings to UTF-8 bytes, compresses with bz2, strips 15-byte header
  - **Dependency**: Add `bzip2` crate to `Cargo.toml`
  - Python: ~5 lines
  - Test values:
    - `('test', 'test')` → `0.08`
    - `('test', 'nani')` → `0.16`
- [ ] **2.8** `LZMANCD` — LZMA binary compressor NCD
  - Converts strings to UTF-8 bytes, compresses with LZMA, strips 14-byte header
  - **Dependency**: Add LZMA crate (e.g., `xz2` or `liblzma`) to `Cargo.toml`
  - Python: ~5 lines
- [ ] **2.9** `ZLIBNCD` — Zlib binary compressor NCD
  - Converts strings to UTF-8 bytes, compresses with zlib, strips 2-byte header
  - **Dependency**: Add `flate2` crate to `Cargo.toml`
  - Python: ~5 lines
- [ ] **2.10** Add unit tests for all compression algorithms

### Phase 3: MongeElkan (DEPENDS on Person 1's `edit_based.rs`)

Implement this **last**. It requires Person 1 to have pushed `DamerauLevenshtein` and `JaroWinkler`.

- [ ] **3.1** `MongeElkan`
  - Imports `DamerauLevenshtein` from `crate::edit_based`
  - Default inner algorithm: `DamerauLevenshtein` with default params
  - For each token in sequence 1, find max similarity against all tokens in sequence 2
  - Optional `symmetric` mode: average over all permutations of sequences
  - `maximum` depends on inner algorithm
  - Python: ~60 lines
  - Test values (using `jaro_winkler` as inner algorithm):
    - `(['Niall'], ['Neal'])` → `0.805`
    - `(['Niall'], ['Nigel'])` → `0.7866666666666667`
  - If Person 1 has not yet pushed `DamerauLevenshtein` or `JaroWinkler`, skip MongeElkan and leave a `// TODO: implement after DamerauLevenshtein is available` comment.

### Phase 4: Final Verification

- [ ] **4.1** `cargo fmt`
- [ ] **4.2** `cargo clippy` — fix all warnings
- [ ] **4.3** `cargo test` — all tests pass
- [ ] **4.4** Document all public APIs with `///` doc comments
- [ ] **4.5** Verify Tversky equivalence with Jaccard (ks=[1,1]) and Sorensen (ks=[0.5,0.5])

---

## Dependencies on Person 1

| What you need | From | When you need it |
|---|---|---|
| Base traits, counter helpers, types, utils | Bootstrap commit | Before any implementation (Phase 0) |
| `DamerauLevenshtein` struct | `src/edit_based.rs` | Before `MongeElkan` only (Phase 3) |
| `JaroWinkler` struct | `src/edit_based.rs` | For `MongeElkan` test cases only (Phase 3) |

Everything except MongeElkan is independent of Person 1's algorithm work.
You can implement 15 out of 16 algorithms immediately after the bootstrap commit.

---

## Testing Responsibilities

You test:

- All algorithms in `token_based.rs` (Jaccard, Sorensen, Tversky, Overlap, Cosine, Tanimoto, MongeElkan, Bag)
- All algorithms in `compression_based.rs` (ArithNCD, RLENCD, BWTRLENCD, SqrtNCD, EntropyNCD, BZ2NCD, LZMANCD, ZLIBNCD)

Test cases MUST cover (per root AGENT.md):

- Empty strings
- Identical strings
- Unicode characters
- ASCII strings
- Different lengths
- Edge cases from Python test files
- Expected outputs matching Python implementation

### Key test patterns from Python tests

**Token-based invariants:**

- Jaccard ≡ Tversky with `ks=[1, 1]` (both `as_set=true` and `as_set=false`)
- Sorensen ≡ Tversky with `ks=[0.5, 0.5]` (both `as_set=true` and `as_set=false`)
- Bag distance of anagrams = 0

**Compression-based invariants:**

- Monotonicity: `alg('test', 'test') <= alg('test', 'text') <= alg('test', 'nani')`
- Symmetry: `distance(a, b) == distance(b, a)`
- `normalized_distance + normalized_similarity == 1`
- Compressor symmetry: `_compress(text) == _compress(reversed(text))` (for entropy)
- Compressor idempotency: `_get_size(text * 2) < _get_size(text) * 2` (for entropy)

---

## Merge Conflict Prevention

| File | Risk | Mitigation |
|---|---|---|
| `Cargo.toml` | **None** | Person 1 is sole owner. Person 2 never edits directly. |
| `src/lib.rs` | **None** | Already contains your module declarations from bootstrap. READ ONLY afterward. |
| `src/token_based.rs` | **None** | You are the sole owner. |
| `src/compression_based.rs` | **None** | You are the sole owner. |
| `src/base.rs` | **None** | READ ONLY. Import from it, never modify. |
| `src/types.rs` | **None** | READ ONLY. Import from it, never modify. |
| `src/utils.rs` | **None** | READ ONLY. Import from it, never modify. |
| `src/edit_based.rs` | **None** | Read-only. Import `DamerauLevenshtein` and `JaroWinkler`. |
| All other `src/*.rs` | **None** | Owned by Person 1. Do not touch. |

---

## Coding Style (per AGENT.md and ./AGENT.md)

- Idiomatic Rust: iterators, slices, generics, traits, enums, Result
- Safe Rust only — no `unsafe`
- Avoid `unwrap()` in library code — use `Option`/`Result` properly
- Prefer slices (`&[T]`) over `Vec` references (`&Vec<T>`)
- Minimize allocations — prefer stack allocation or borrowing
- No unnecessary cloning
- No premature optimization — correctness first
- Use `cargo fmt`, `cargo clippy`, `cargo test` before considering work complete
- Prefer small commits — one algorithm per commit is ideal
- If you need a crate (e.g., for bz2/zlib/lzma compression), justify the dependency

---

## When to Stop and Ask for Clarification

- If the bootstrap commit has not been pushed yet — wait for Person 1
- If `base.rs` does not provide the counter helpers you need (intersect, union, count) — open an issue
- If `DamerauLevenshtein` or `JaroWinkler` is not yet available and you need to implement MongeElkan — skip MongeElkan, leave a TODO
- If you need to add an external Rust crate dependency — justify it (compression crates are acceptable)
- If `ArithNCD` requires exact rational arithmetic and you're unsure whether to add `num-rational` — ask
- If the BZ2/LZMA/Zlib header stripping behavior differs between Python and Rust compression libraries — ask
- If a Python test expects a value that depends on Python-specific compression internals — ask

---

## Assumptions You Can Make

- Rust stable toolchain is available
- Person 1 will provide base traits with: `distance`, `similarity`, `maximum`, `normalized_distance`, `normalized_similarity` methods
- Person 1 will provide counter helpers: `get_counters`, `intersect_counters`, `union_counters`, `sum_counters`, `count_counters`
- The `external` parameter from Python (for delegating to C libraries) is NOT ported — it's Python-specific
- The `qval` parameter should be supported for token-based algorithms
- The `as_set` parameter should be supported for token-based algorithms
- Vector-based algorithms (`vector_based.py`) are **draft/incomplete** in Python and are NOT in scope
- Binary NCD algorithms (BZ2, LZMA, Zlib) may require external crates — this is acceptable
- Person 1 will not modify `token_based.rs` or `compression_based.rs` after bootstrap

---

## Algorithm Quick-Reference

### Token-Based (all use counter operations from `base.rs`)

| Algorithm | Formula | Metric Type | Python LOC |
|---|---|---|---|
| Jaccard | `\|A∩B\| / \|A∪B\|` | Similarity | ~35 |
| Sorensen | `2·\|A∩B\| / (\|A\|+\|B\|)` | Similarity | ~30 |
| Tversky | `\|A∩B\| / (\|A∩B\| + α·\|A\B\| + β·\|B\A\|)` | Similarity | ~55 |
| Overlap | `\|A∩B\| / min(\|A\|,\|B\|)` | Similarity | ~35 |
| Cosine | `\|A∩B\| / (\|A\|·\|B\|)^(1/N)` | Similarity | ~35 |
| Tanimoto | `log₂(Jaccard)` | Similarity | ~10 |
| MongeElkan | token-level max similarity | Similarity | ~60 |
| Bag | `max(\|A\(A∩B)\|, \|B\(A∩B)\|)` | Distance | ~5 |

### Compression-Based (all use NCD formula)

| Algorithm | Compressor | Needs Crate? | Python LOC |
|---|---|---|---|
| RLENCD | Run-length encoding | No | ~15 |
| BWTRLENCD | BWT + RLE | No | ~20 |
| SqrtNCD | √count sum | No | ~15 |
| EntropyNCD | Shannon entropy | No | ~30 |
| ArithNCD | Arithmetic coding | Maybe (`num-rational`) | ~70 |
| BZ2NCD | BZ2 compression | Yes (`bzip2`) | ~5 |
| LZMANCD | LZMA compression | Yes (`xz2` or similar) | ~5 |
| ZLIBNCD | Zlib compression | Yes (`flate2`) | ~5 |

---

## Checklist Summary

### Token-Based (Phase 1)

- [x] `src/token_based.rs` — Jaccard
- [x] `src/token_based.rs` — Sorensen
- [x] `src/token_based.rs` — Overlap
- [x] `src/token_based.rs` — Cosine
- [x] `src/token_based.rs` — Tversky
- [x] `src/token_based.rs` — Tanimoto
- [x] `src/token_based.rs` — Bag
- [x] Tversky ↔ Jaccard equivalence tested (ks=[1,1])
- [x] Tversky ↔ Sorensen equivalence tested (ks=[0.5,0.5])

### Compression-Based (Phase 2)

- [x] `src/compression_based.rs` — NCD base implementation
- [x] `src/compression_based.rs` — RLENCD
- [x] `src/compression_based.rs` — BWTRLENCD
- [x] `src/compression_based.rs` — SqrtNCD
- [x] `src/compression_based.rs` — EntropyNCD
- [x] `src/compression_based.rs` — ArithNCD
- [x] `src/compression_based.rs` — BZ2NCD
- [x] `src/compression_based.rs` — LZMANCD
- [x] `src/compression_based.rs` — ZLIBNCD

### MongeElkan (Phase 3 — after Person 1 pushes DamerauLevenshtein)

- [ ] `src/token_based.rs` — MongeElkan *(blocked: waiting on Person 1's `DamerauLevenshtein` + `JaroWinkler` in `src/edit_based.rs`)*

### Final Verification (Phase 4)

- [x] All unit tests pass
- [x] `cargo fmt` passes
- [x] `cargo clippy` passes
- [x] Public APIs documented with `///` doc comments
- [x] External crate dependencies justified and minimal
