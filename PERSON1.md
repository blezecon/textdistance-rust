# PERSON 1 — Assignment: Infrastructure Bootstrap + Edit-Based + Sequence-Based + Phonetic + Simple

---

## Collaboration Rules

Before implementing any code:

- Read the repository root [`AGENT.md`](file:///home/blezecon/Code/textdistance-rust/AGENT.md).
- Read [`rust-port/AGENT.md`](file:///home/blezecon/Code/textdistance-rust/rust-port/AGENT.md).
- Read BOTH [`PERSON1.md`](file:///home/blezecon/Code/textdistance-rust/rust-port/PERSON1.md) and [`PERSON2.md`](file:///home/blezecon/Code/textdistance-rust/rust-port/PERSON2.md).

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

If shared infrastructure changes are required after the bootstrap phase, coordinate with Person 2 before modifying any shared file. Leave a TODO or open an issue instead of implementing another person's assignment.

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

You have two responsibilities:

1. **Bootstrap** — create the shared project scaffold in a single initial commit so both developers can work in parallel immediately afterward.
2. **Algorithm implementation** — edit-based, sequence-based, phonetic, and simple algorithms.

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
├── src/token_based.rs ─────────── Person 2
│   └── MongeElkan depends on:
│       ├── edit_based::DamerauLevenshtein (Person 1)
│       └── edit_based::JaroWinkler (Person 1, for tests only)
│
└── src/compression_based.rs ───── Person 2
```

Key cross-module dependency: `MongeElkan` (Person 2, `token_based.rs`) imports `DamerauLevenshtein` from `edit_based.rs` (Person 1). Person 2 should implement MongeElkan last.

---

## Bootstrap Timeline

### Phase 0 — Bootstrap (Person 1 only)

- Person 1 creates the shared infrastructure in a **single commit**.
- This commit includes: `Cargo.toml`, `src/lib.rs`, `src/base.rs`, `src/types.rs`, `src/utils.rs`, and **empty stub files** for every algorithm module.
- `cargo check`, `cargo fmt`, and `cargo clippy` must pass.
- Person 1 commits and pushes.
- Person 2 pulls the latest changes.

### Phase 1 — Parallel Development (both developers)

- Both developers work independently on their own files.
- No coordination needed except for the MongeElkan dependency (see below).
- Neither developer modifies `lib.rs`, `Cargo.toml`, `base.rs`, `types.rs`, or `utils.rs` after bootstrap.

### Phase 2 — Integration

- Both developers run `cargo test` on the merged codebase.
- Fix any integration issues.
- Final `cargo fmt` + `cargo clippy` pass.

---

## File Ownership

Every file has exactly one owner. Shared files are only modified during bootstrap.

| File | Owner | After Bootstrap |
|---|---|---|
| `rust-port/Cargo.toml` | Person 1 | **Sole Owner: Person 1** — Person 2 MUST NOT modify Cargo.toml directly. |
| `rust-port/src/lib.rs` | Person 1 (bootstrap) | **READ ONLY / Frozen** — neither developer nor AI assistant modifies after bootstrap unless fixing a critical bug. |
| `rust-port/src/base.rs` | Person 1 (bootstrap) | **READ ONLY / Frozen** — neither developer nor AI assistant modifies after bootstrap unless fixing a critical bug. |
| `rust-port/src/types.rs` | Person 1 (bootstrap) | **READ ONLY / Frozen** — neither developer nor AI assistant modifies after bootstrap unless fixing a critical bug. |
| `rust-port/src/utils.rs` | Person 1 (bootstrap) | **READ ONLY / Frozen** — neither developer nor AI assistant modifies after bootstrap unless fixing a critical bug. |
| `rust-port/src/simple.rs` | Person 1 | Person 1 only. |
| `rust-port/src/edit_based.rs` | Person 1 | Person 1 only. Person 2 imports from it. |
| `rust-port/src/sequence_based.rs` | Person 1 | Person 1 only. |
| `rust-port/src/phonetic.rs` | Person 1 | Person 1 only. |
| `rust-port/src/token_based.rs` | Person 2 | Person 2 only. Person 1 creates empty stub during bootstrap. |
| `rust-port/src/compression_based.rs` | Person 2 | Person 2 only. Person 1 creates empty stub during bootstrap. |

---

## Scope Boundaries

### You MUST touch

- `rust-port/Cargo.toml` — bootstrap only
- `rust-port/src/lib.rs` — bootstrap only
- `rust-port/src/base.rs` — bootstrap only
- `rust-port/src/types.rs` — bootstrap only
- `rust-port/src/utils.rs` — bootstrap only
- `rust-port/src/simple.rs` — your algorithm module
- `rust-port/src/edit_based.rs` — your algorithm module
- `rust-port/src/sequence_based.rs` — your algorithm module
- `rust-port/src/phonetic.rs` — your algorithm module

### You MUST NOT touch (after bootstrap)

- `rust-port/src/token_based.rs` — owned by Person 2
- `rust-port/src/compression_based.rs` — owned by Person 2
- Any Python files under `textdistance/` — read-only reference
- Any files outside `rust-port/`

---

## Reference Python Files

Read these before implementing. They are the specification:

| Rust module | Python reference |
|---|---|
| `base.rs` | [`textdistance/algorithms/base.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/base.py) |
| `types.rs` | [`textdistance/algorithms/types.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/types.py) |
| `utils.rs` | [`textdistance/utils.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/utils.py) |
| `edit_based.rs` | [`textdistance/algorithms/edit_based.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/edit_based.py) |
| `sequence_based.rs` | [`textdistance/algorithms/sequence_based.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/sequence_based.py) |
| `phonetic.rs` | [`textdistance/algorithms/phonetic.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/phonetic.py) |
| `simple.rs` | [`textdistance/algorithms/simple.py`](file:///home/blezecon/Code/textdistance-rust/textdistance/algorithms/simple.py) |

---

## Python Test Files (for expected values)

| Test directory | Algorithms covered |
|---|---|
| [`tests/test_edit/`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/) | Hamming, Levenshtein, DamerauLevenshtein, Jaro, JaroWinkler, StrCmp95, NeedlemanWunsch, Gotoh, SmithWaterman, MLIPNS, Matrix, Editex |
| [`tests/test_sequence/`](file:///home/blezecon/Code/textdistance-rust/tests/test_sequence/) | LCSSeq, LCSStr |
| [`tests/test_phonetic/`](file:///home/blezecon/Code/textdistance-rust/tests/test_phonetic/) | Editex |
| [`tests/test_common.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_common.py) | Common interface tests for all algorithms |

---

## Recommended Implementation Order

### Phase 0: Bootstrap Commit (BLOCKING — do this first)

Create all shared infrastructure in a single commit. Person 2 cannot start until this is pushed.

- [ ] **0.1** Create `Cargo.toml`
  - Name: `textdistance`
  - Edition: 2021
  - No external dependencies initially
- [ ] **0.2** Create `src/types.rs`
  - Define `SimFunc` as `Option<fn(&T, &T) -> f64>` or equivalent trait-based approach
  - Define `TestFunc` as `Option<fn(&T, &T) -> bool>` or equivalent trait-based approach
  - Python reference: `SimFunc = Optional[Callable[[T, T], float]]`, `TestFunc = Optional[Callable[[T, T], bool]]`
- [ ] **0.3** Create `src/utils.rs`
  - Implement `find_ngrams<T: Clone>(input: &[T], n: usize) -> Vec<Vec<T>>`
  - Python reference: `find_ngrams(input_list, n)` returns `list(zip(*[input_list[i:] for i in range(n)]))`
- [ ] **0.4** Create `src/base.rs` — the core trait hierarchy
  - Define a trait (e.g., `DistanceMetric`) with methods:
    - `fn distance(&self, s1: &[T], s2: &[T]) -> f64`
    - `fn similarity(&self, s1: &[T], s2: &[T]) -> f64`
    - `fn maximum(&self, s1: &[T], s2: &[T]) -> f64`
    - `fn normalized_distance(&self, s1: &[T], s2: &[T]) -> f64`
    - `fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> f64`
  - Provide default implementations matching Python:
    - `similarity = maximum - distance` (for distance-first algorithms)
    - `distance = maximum - similarity` (for similarity-first algorithms)
    - `normalized_distance = distance / maximum` (returns 0 if maximum is 0)
    - `normalized_similarity = 1 - normalized_distance`
  - Implement helper functions:
    - `is_ident(s1, s2) -> bool` — check if two sequences are identical
    - `quick_answer_distance(s1, s2) -> Option<f64>` — short-circuit for empty/identical
    - `quick_answer_similarity(s1, s2, maximum) -> Option<f64>` — similarity variant
  - Implement `get_sequences(s1, s2, qval)` for q-gram tokenization
  - Implement Counter-based helpers needed by Person 2's token-based algorithms:
    - `get_counters` — convert sequences to frequency maps
    - `intersect_counters` — Counter intersection (`&` in Python)
    - `union_counters` — Counter union (`|` in Python)
    - `sum_counters` — Counter sum (`+` in Python)
    - `count_counters(counter, as_set) -> usize` — count elements (unique if as_set, total otherwise)
  - **Design note**: Use traits, not struct inheritance. Use a trait `DistanceMetric` with provided (default) methods where Python uses `Base`, and a separate trait or marker for `SimilarityMetric` where Python uses `BaseSimilarity`.
- [ ] **0.5** Create `src/lib.rs` with ALL module declarations:
  ```rust
  pub mod base;
  pub mod types;
  pub mod utils;
  pub mod simple;
  pub mod edit_based;
  pub mod sequence_based;
  pub mod phonetic;
  pub mod token_based;
  pub mod compression_based;
  ```
- [ ] **0.6** Create empty stub files for ALL algorithm modules:
  - `src/simple.rs` — empty file (or `// TODO: Person 1`)
  - `src/edit_based.rs` — empty file (or `// TODO: Person 1`)
  - `src/sequence_based.rs` — empty file (or `// TODO: Person 1`)
  - `src/phonetic.rs` — empty file (or `// TODO: Person 1`)
  - `src/token_based.rs` — empty file (or `// TODO: Person 2`)
  - `src/compression_based.rs` — empty file (or `// TODO: Person 2`)
- [ ] **0.7** Verify: `cargo check` passes, `cargo fmt` passes, `cargo clippy` passes
- [ ] **0.8** **Commit and push**. Notify Person 2 that bootstrap is complete.

> **IMPORTANT**: After this commit, `lib.rs` should NEVER need modification again.
> All module declarations already exist. Both developers populate their own empty stub files.
> This eliminates all `lib.rs` merge conflicts.

### Phase 1: Simple Algorithms

These are trivial and exercise the base trait.

- [ ] **1.1** Implement in `src/simple.rs`:
  - `Prefix` — common prefix similarity (uses `_get_sequences`, `sim_test`)
  - `Postfix` — common postfix similarity (reverses inputs, delegates to Prefix)
  - `Length` — absolute length difference: `max(lengths) - min(lengths)`
  - `Identity` — returns 1 if identical, 0 otherwise; `maximum` is always 1
  - `Matrix` — lookup similarity from a hashmap; fallback to match/mismatch costs
- [ ] **1.2** Add unit tests in `src/simple.rs`

### Phase 2: Edit-Based Algorithms

The largest module. Implement one algorithm at a time, per AGENT.md.

- [ ] **2.1** `Hamming`
  - Counts mismatched positions
  - Supports `truncate` option (zip vs zip_longest behavior)
  - Supports custom `test_func`
  - Python: ~30 lines
- [ ] **2.2** `Levenshtein`
  - Iterative 2-row DP implementation (`_cycled` method)
  - Do NOT implement the recursive version (it's not used in practice)
  - Supports custom `test_func`
  - Python: ~75 lines
- [ ] **2.3** `DamerauLevenshtein`
  - Two modes: `restricted` (OSA) and `unrestricted`
  - Restricted: DP with dict-based matrix (`_pure_python_restricted`)
  - Unrestricted: Wikipedia algorithm with `da` dictionary (`_pure_python_unrestricted`)
  - Supports custom `test_func`
  - Python: ~155 lines
  - **IMPORTANT**: Person 2's `MongeElkan` imports `DamerauLevenshtein`. Export it as `pub struct` in your module's public API.
- [ ] **2.4** `Jaro` and `JaroWinkler`
  - `JaroWinkler` is the full implementation. `Jaro` is `JaroWinkler` with `winklerize=false`.
  - Character window matching, transposition counting, prefix boosting
  - `long_tolerance` option for long strings
  - `maximum` is always 1
  - Python: ~120 lines
  - **IMPORTANT**: Person 2's `MongeElkan` test cases use `JaroWinkler`. Export it as `pub struct`.
- [ ] **2.5** `StrCmp95`
  - Similar to Jaro-Winkler but with phonetic/character-recognition partial credit matrix (`sp_mx`)
  - 36-entry similarity matrix for visually/phonetically similar characters (e.g., 'O' ↔ '0')
  - Converts to uppercase
  - `maximum` is always 1
  - Python: ~150 lines
- [ ] **2.6** `NeedlemanWunsch`
  - Global sequence alignment with 2D DP matrix
  - Custom `gap_cost` and `sim_func`
  - Overrides `normalized_distance` and `normalized_similarity` with custom formulas involving `minimum()`
  - `distance = -1 * similarity`
  - Python requires numpy — implement using pure Rust `Vec<Vec<f64>>`
  - Python: ~85 lines
- [ ] **2.7** `Gotoh`
  - Extension of NeedlemanWunsch with affine gap penalties
  - Uses 3 DP matrices (d_mat, p_mat, q_mat)
  - `gap_open` and `gap_ext` parameters
  - Python requires numpy — implement using pure Rust `Vec<Vec<f64>>`
  - Python: ~80 lines
- [ ] **2.8** `SmithWaterman`
  - Local sequence alignment with 2D DP matrix
  - Like NeedlemanWunsch but clamps to `max(0, ...)`
  - `maximum = min(len(s1), len(s2))`
  - Python requires numpy — implement using pure Rust `Vec<Vec<f64>>`
  - Python: ~55 lines
- [ ] **2.9** `MLIPNS`
  - Uses `Hamming` distance internally
  - Iteratively compares threshold against mismatches
  - `maximum` is always 1
  - Python: ~45 lines
- [ ] **2.10** Add unit tests for each edit-based algorithm. Use test values from:
  - [`tests/test_edit/test_hamming.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_hamming.py)
  - [`tests/test_edit/test_levenshtein.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_levenshtein.py)
  - [`tests/test_edit/test_damerau_levenshtein.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_damerau_levenshtein.py)
  - [`tests/test_edit/test_jaro.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_jaro.py)
  - [`tests/test_edit/test_jaro_winkler.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_jaro_winkler.py)
  - [`tests/test_edit/test_strcmp95.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_strcmp95.py)
  - [`tests/test_edit/test_needleman_wunsch.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_needleman_wunsch.py)
  - [`tests/test_edit/test_gotoh.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_gotoh.py)
  - [`tests/test_edit/test_smith_waterman.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_smith_waterman.py)
  - [`tests/test_edit/test_mlipns.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_mlipns.py)

### Phase 3: Sequence-Based Algorithms

- [ ] **3.1** `LCSSeq` — Longest Common Subsequence
  - DP table implementation for 2 sequences (`_dynamic` method)
  - Returns the subsequence itself; `similarity()` returns its length
  - Uses `find_ngrams` from `utils.rs` via `_get_sequences`
  - Python: ~100 lines
- [ ] **3.2** `LCSStr` — Longest Common Substring
  - For 2 short strings: implement SequenceMatcher-equivalent (find longest match)
  - For N strings or long strings: n-gram search approach (`_custom` method)
  - Python: ~40 lines
- [ ] **3.3** `RatcliffObershelp` — Gestalt Pattern Matching
  - Recursive: find LCSStr, then recurse on left and right remainders
  - `maximum` is always 1
  - Uses `LCSStr` internally
  - Python: ~40 lines
- [ ] **3.4** Add unit tests using values from:
  - [`tests/test_sequence/test_lcsseq.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_sequence/test_lcsseq.py)
  - [`tests/test_sequence/test_lcsstr.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_sequence/test_lcsstr.py)

### Phase 4: Phonetic Algorithms

- [ ] **4.1** `MRA` — Match Rating Approach
  - `_calc_mra`: uppercase, strip non-leading vowels, remove consecutive duplicates, truncate to 6 chars
  - Compares transformed sequences by removing matched chars iteratively
  - Python: ~50 lines
- [ ] **4.2** `Editex` — Phonetic edit distance
  - 10 predefined letter groups (AEIOUY, BP, CKQ, DT, LR, MN, GJ, FPV, SXZ, CSZ)
  - `ungrouped` = {H, W}
  - Cost functions: `r_cost` (replacement), `d_cost` (deletion)
  - 2D DP table similar to Levenshtein but with phonetic costs
  - `local` mode skips first-column initialization
  - Python: ~100 lines
- [ ] **4.3** Add unit tests using values from:
  - [`tests/test_phonetic/test_editex.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_phonetic/test_editex.py)
  - [`tests/test_edit/test_editex.py`](file:///home/blezecon/Code/textdistance-rust/tests/test_edit/test_editex.py)

### Phase 5: Final Verification

- [ ] **5.1** `cargo fmt`
- [ ] **5.2** `cargo clippy` — fix all warnings
- [ ] **5.3** `cargo test` — all tests pass
- [ ] **5.4** Document all public APIs with `///` doc comments
- [ ] **5.5** Verify `DamerauLevenshtein` and `JaroWinkler` are exported and usable from `token_based.rs`

---

## Dependencies on Person 2

- **None**. All your modules are independent of Person 2's work.
- Person 2 will import your `DamerauLevenshtein` and `JaroWinkler` for `MongeElkan`.
- You do not need to wait for Person 2 at any point.

---

## Testing Responsibilities

You test:

- All algorithms in `simple.rs`
- All algorithms in `edit_based.rs`
- All algorithms in `sequence_based.rs`
- All algorithms in `phonetic.rs`
- Base trait default method behavior (in `base.rs`)
- Utility functions in `utils.rs` (`find_ngrams`)

Test cases MUST cover (per root AGENT.md):

- Empty strings
- Identical strings
- Unicode characters
- ASCII strings
- Different lengths
- Edge cases from Python test files
- Expected outputs matching Python implementation

---

## Merge Conflict Prevention

| File | Risk | Mitigation |
|---|---|---|
| `Cargo.toml` | **None** | Person 1 is sole owner. Person 2 never edits directly. |
| `src/lib.rs` | **None** | Created during bootstrap. Frozen / READ ONLY afterward. |
| `src/base.rs` | **None** | Created during bootstrap. Frozen / READ ONLY afterward. |
| `src/types.rs` | **None** | Created during bootstrap. Frozen / READ ONLY afterward. |
| `src/utils.rs` | **None** | Created during bootstrap. Frozen / READ ONLY afterward. |
| `src/simple.rs` | **None** | You are the sole owner. |
| `src/edit_based.rs` | **None** | You are the sole owner. Person 2 imports from it. |
| `src/sequence_based.rs` | **None** | You are the sole owner. |
| `src/phonetic.rs` | **None** | You are the sole owner. |
| `src/token_based.rs` | **None** | Empty stub from bootstrap. Person 2 owns it. |
| `src/compression_based.rs` | **None** | Empty stub from bootstrap. Person 2 owns it. |

---

## Coding Style (per AGENT.md and rust-port/AGENT.md)

- Idiomatic Rust: iterators, slices, generics, traits, enums, Result
- Safe Rust only — no `unsafe`
- Avoid `unwrap()` in library code — use `Option`/`Result` properly
- Prefer slices (`&[T]`) over `Vec` references (`&Vec<T>`)
- Minimize allocations — prefer stack allocation or borrowing
- No unnecessary cloning
- No premature optimization — correctness first
- Use `cargo fmt`, `cargo clippy`, `cargo test` before considering work complete
- Prefer small commits — one algorithm per commit is ideal
- Do NOT introduce unnecessary external dependencies

---

## When to Stop and Ask for Clarification

- If the Python implementation uses `numpy` and you're unsure how to translate a specific matrix operation
- If a Python test expects a value that seems incorrect or ambiguous
- If you need to add an external Rust crate dependency (keep dependencies minimal)
- If the trait design for `Base`/`BaseSimilarity` needs a different approach than described
- If you discover that `NeedlemanWunsch.normalized_similarity` has different behavior than standard normalization (it does — it uses a custom formula)
- If you are unsure whether a function should be `pub` or `pub(crate)`

---

## Assumptions You Can Make

- Rust stable toolchain is available
- No external Rust crates are needed for edit-based, sequence-based, phonetic, or simple algorithms
- The `external` parameter from Python (for delegating to C libraries) is NOT ported — it's Python-specific
- The `qval` parameter should be supported but you can simplify by accepting `&str` and handling q-gram splitting internally
- Vector-based algorithms (`vector_based.py`) are **draft/incomplete** in Python and are NOT in scope for either developer
- Person 2 will not modify any file you own

---

## Checklist Summary

### Bootstrap (Phase 0)

- [ ] `Cargo.toml` created
- [ ] `src/lib.rs` created with ALL 9 module declarations
- [ ] `src/types.rs` implemented
- [ ] `src/utils.rs` implemented (`find_ngrams`)
- [ ] `src/base.rs` implemented (traits + counter helpers)
- [ ] Empty stub files for all 6 algorithm modules created
- [ ] `cargo check` passes
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [ ] Committed and pushed

### Algorithms (Phases 1–4)

- [ ] `src/simple.rs` — Prefix
- [ ] `src/simple.rs` — Postfix
- [ ] `src/simple.rs` — Length
- [ ] `src/simple.rs` — Identity
- [ ] `src/simple.rs` — Matrix
- [ ] `src/edit_based.rs` — Hamming
- [ ] `src/edit_based.rs` — Levenshtein
- [ ] `src/edit_based.rs` — DamerauLevenshtein (restricted + unrestricted)
- [ ] `src/edit_based.rs` — Jaro
- [ ] `src/edit_based.rs` — JaroWinkler
- [ ] `src/edit_based.rs` — StrCmp95
- [ ] `src/edit_based.rs` — NeedlemanWunsch
- [ ] `src/edit_based.rs` — Gotoh
- [ ] `src/edit_based.rs` — SmithWaterman
- [ ] `src/edit_based.rs` — MLIPNS
- [ ] `src/sequence_based.rs` — LCSSeq
- [ ] `src/sequence_based.rs` — LCSStr
- [ ] `src/sequence_based.rs` — RatcliffObershelp
- [ ] `src/phonetic.rs` — MRA
- [ ] `src/phonetic.rs` — Editex

### Final Verification (Phase 5)

- [ ] All unit tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [ ] Public APIs documented with `///` doc comments
- [ ] `DamerauLevenshtein` and `JaroWinkler` are `pub` and importable
