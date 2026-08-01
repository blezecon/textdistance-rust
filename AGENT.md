# AGENT.md

# Project

This repository is an independent Rust implementation of the Python
**TextDistance** library.

The goal is to faithfully port the original implementation to Rust for a
hackathon while preserving correctness, behavior, and API semantics whenever
practical.

The original Python project is the specification.

---

# Source of Truth

Always treat the Python implementation as the canonical reference.

Before implementing any algorithm:

1. Read the corresponding Python source.
2. Read the corresponding Python tests.
3. Understand the algorithm.
4. Explain the Python implementation in 3–5 sentences.
5. Only then begin writing Rust code.

If Rust requires a different implementation technique, preserve observable
behavior instead of copying the implementation literally.

Compatibility is more important than cleverness.

---

# Project Goals

Priority order:

1. Correctness
2. Python compatibility
3. Complete algorithm coverage
4. Idiomatic Rust
5. Performance
6. Minimal dependencies

Never sacrifice correctness for performance.

---

# Implementation Workflow

Implement **exactly one algorithm at a time**.

For every algorithm:

1. Read the Python implementation.
2. Explain the implementation.
3. Implement the Rust version.
4. Add unit tests.
5. Compare behavior against Python.
6. Run:

```bash
cargo fmt
cargo clippy
cargo test
```

7. Stop.

Do not automatically continue to the next algorithm.

Wait for the next instruction.

---

# Definition of Done

An algorithm is considered complete only when:

- Rust implementation compiles.
- Behavior matches Python.
- Public API is documented where appropriate.
- Unit tests have been added.
- Existing tests continue to pass.
- cargo fmt passes.
- cargo clippy passes.
- cargo test passes.
- No unrelated files were modified.

---

# Code Style

Use idiomatic Rust.

Prefer:

- traits
- enums
- iterators
- slices
- generics
- borrowing

Avoid:

- unnecessary allocations
- unnecessary cloning
- unsafe
- over-engineering
- premature optimization

Prefer readability over clever code.

---

# API Compatibility

Where practical:

- Preserve algorithm names.
- Preserve parameter names.
- Preserve default values.
- Preserve return values.
- Preserve normalization behavior.
- Preserve edge-case behavior.

If exact compatibility is impossible, document the difference.

---

# Dependencies

Keep dependencies minimal.

Before adding a crate:

- verify the standard library cannot solve the problem
- explain why the dependency is needed
- choose actively maintained crates

Do not add dependencies for convenience alone.

---

# Testing

Every implementation should include tests.

Tests should cover:

- empty input
- identical input
- Unicode
- ASCII
- different lengths
- edge cases
- examples from Python tests
- normalization behavior

When practical, compare outputs against the original Python implementation.

---

# Performance

Correctness comes first.

Optimize only after:

- implementation is correct
- tests pass
- behavior matches Python

Avoid micro-optimizations until correctness is established.

---

# Documentation

Document all public types.

Document public functions.

Include examples where useful.

Document any intentional differences from Python.

---

# Commit Guidelines

Prefer small commits.

Ideally:

- one algorithm
- corresponding tests
- documentation updates

Example commit messages:

```
feat(edit): implement Levenshtein
test(edit): add Levenshtein tests
docs(edit): document Levenshtein
```

---

# Architecture Rules

This repository is a port.

It is **not** a redesign.

Do NOT:

- redesign the library
- invent new APIs
- merge unrelated algorithms
- rewrite working code for style
- change algorithm behavior for performance

Compatibility is more important than architectural perfection.

---

# Ambiguous Python Code

The Python implementation is the specification.

If the Python implementation is:

- incomplete
- ambiguous
- marked TODO
- intentionally unfinished
- dependent on Python-specific behavior

then:

- do not invent behavior
- explain the ambiguity
- identify the affected Python source
- stop and ask for clarification

---

# AI Workflow

When working in this repository:

- Read this AGENT.md before making changes.
- Understand the relevant Python implementation before writing Rust.
- Modify only files required for the current task.
- Avoid unrelated refactors.
- Complete exactly one algorithm per task unless explicitly instructed otherwise.
- Stop after completing the requested work.

Do not continue implementing additional algorithms without being asked.

---

# Guiding Principle

If there is ever a conflict between:

- writing "more Rust-like" code, or
- preserving the behavior of the original Python implementation,

choose compatibility with the Python implementation.