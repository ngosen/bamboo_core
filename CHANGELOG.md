# Changelog

All notable changes to this project will be documented in this file.

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
