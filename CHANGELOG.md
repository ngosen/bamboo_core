# Changelog

All notable changes to this project will be documented in this file.

## [0.3.24] - 2026-09-14

### Performance & Memory Architecture
- **Zero-Allocation Hot Path:** Stack-allocated active syllable composition (`[Transformation; MAX_ACTIVE_TRANS]`, where `MAX_ACTIVE_TRANS = 16`), achieving strictly 0 heap allocations during typing.
- **Cache-Optimized Transformation:** Aligned `Transformation` to exactly 16 bytes (128-bit boundary) with a static compile-time size assertion.
- **Zero-Allocation Output:** Cached preedit buffer enables `output_str(&self) -> &str` and `output(&self) -> Cow<'_, str>` without heap allocations.
- **Counting Sort Rule Partitioning:** Precomputed `EngineRules` with Counting Sort into direct-indexed ASCII slices (`[(u16, u16); 128]`) and partitioned non-ASCII rules, eliminating hot-path linear rule scans.
- **Zero-Copy Engine Preset Sharing:** Built-in presets (`Telex`, `VNI`, `VIQR`, etc.) now share rule tables via `Arc<EngineRules>` and `Arc<InputMethod>` stored in `LazyLock`, reducing `Engine` stack size from 6,176 B to 5,360 B (5.2 KB) and making `Engine::from_preset()` instantaneous with zero heap copying.
- **Sub-Microsecond Keystroke Latency:** Measured single-word latency down to ~140 ns ('vietj' -> 'việt') and full 27-character sentence streams at ~1.3 µs (8.7x faster than skey, up to 16x faster on paragraphs).

### Safety & FFI Invariants
- **Strict Safe FFI Boundaries:** Enforced `#![deny(unsafe_op_in_unsafe_fn)]` across `ffi.rs`.
- **Defensive Pointer Validation:** All FFI entrypoints validate pointers (`engine.as_mut()`) and catch panics gracefully with explicit `// SAFETY:` rationale comments.

### Domain-Driven Modularization
- **Clean Architecture:** Refactored monolithic flat source files into cohesive domain modules:
  - `src/engine/`: `mod.rs`, `rules.rs`, `snapshot.rs`, `state.rs`, `restore.rs`.
  - `src/input_method/`: `mod.rs`, `preset.rs`, `rule.rs`, `definitions.rs`.
  - `src/orthography/`: `mod.rs`, `phonetics.rs`, `spelling.rs`, `syllable.rs`.
  - `src/encoder/`: `mod.rs`, `tables.rs` (isolated 45 KB legacy charset definitions).
  - `src/dfa/`: `mod.rs`, `flattener.rs`.
- **100% Backward Compatibility:** All public API surfaces and `bamboo_core::advanced` re-exports preserved without breaking changes.
- **Clean Documentation & Clippy:** 0 Clippy warnings under strict lints, 8/8 rustdoc doctests passing, and clean `cargo doc` output.

## [0.3.23] - 2026-09-02

### Documentation & docs.rs
- **Comprehensive API Documentation:** Reached 100% rustdoc coverage under `#![warn(missing_docs)]` with zero warnings.
- **Enhanced Crate-Level Docs:** Added API selection guide, architecture overview, and complete doctests for real-time IME integration, 3-way diff, and dual backspace modes.
- **Detailed Module Docs:** Thorough documentation and runnable examples for `Config`, `ConfigBuilder`, `Dfa` (bitset/SWAR/arena internals), `encoder` (16 legacy charsets), and C-FFI safety contracts.
- **Docs.rs Metadata:** Configured `[package.metadata.docs.rs]` in `Cargo.toml`.

## [0.3.22] - 2026-08-24

### Safety & Code Quality
- **Strict Clippy Lints:** Added lint rules enforcing safety comments on unsafe blocks, doc backtick formatting, and avoiding anti-patterns (`clippy::undocumented_unsafe_blocks`, `clippy::doc_markdown`, `clippy::manual_let_else`, `clippy::semicolon_if_nothing_returned`, `clippy::match_same_arms`).
- **Documented Unsafe Blocks:** Added thorough `SAFETY` invariants documentation across DFA and flattener modules.
- **Contextual Invariant Checks:** Replaced bare `unwrap()` with explicit `.expect()` documentation in 8-byte word LCP comparison.
- **FFI Robustness:** Added safe mutex error handling preventing panics across FFI boundaries and added `#[repr(C)] pub enum BambooMethod``.

### Performance & Memory
- **Zero-Allocation Output (`Cow<str>`):** `Engine::output()` and `Engine::get_processed_str_cow()` return `Cow<'_, str>`, returning borrowed string slices for empty or uncommitted buffers without heap allocation.
- **Array-Based ASCII Rule Indexing:** Replaced `BTreeMap` key lookup in `Engine::with_config` with direct `[Vec<Rule>; 128]` array indexing, eliminating tree node allocations and O(log n) overhead.
- **Zero-Allocation Charset Name Iteration:** Added `charset_names() -> impl Iterator<Item = &'static str>` avoiding temporary `Vec<String>` allocations.
- **Static String Slices in Input Methods:** Changed `InputMethod::name` to `&'static str` avoiding heap strings for built-in methods.

### API & Design Improvements
- **Fluent Config Builder:** Added `ConfigBuilder` with fluent configuration methods (`Config::builder().free_tone_marking(...).build()`).
- **Semantic Enums:** Added `RestoreMark` enum (`RestoreMark::Yes`, `RestoreMark::No`) with seamless `From<bool>` / `Into<RestoreMark>` conversion for `remove_last_char`.
- **Public Getters:** Added accessor methods on `InputMethod` (`rules()`, `super_keys()`, `tone_keys()`, `appending_keys()`, `keys()`).

## [0.3.21] - 2026-08-19
