# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Bug Fixes
- **Tone keys after an invalid word:** a key now edits only the part of the word after the last point where it stopped being a valid syllable, as in the Go core. A tone key after an invalid word is typed as a letter (`enlf` gives `enlf`, not `ènl`), and a tone stays on the syllable it was typed on (`mymfyk` gives `mỳmyk`, not `mymyk`).

## [0.3.25] - 2026-09-25

### Performance & Memory
- **Compact Snapshot Stack:** `Snapshot` shrunk from 272 B to 8 B (`state_id` + `upper_mask` + `active_len` + flags), cutting the `Engine` footprint from ~5.3 KB to ~1.3 KB; the English bypass buffer is now lazily boxed.
- **Cache-Line DFA States:** `Dfa` state reduced from 160 B to 88 B so hot fields fit a single 64 B cache line, with `u16` state IDs.
- **Flattened Output Caching:** DFA states cache flattened lowercase output (`flat_arena`); a DFA hit becomes memcpy + case-apply instead of full effect-chain resolution.
- **Dense Phonetics Tables:** `MARK_TABLE [[char; 5]; 72]` replaces 4 PHF lookups per `add_mark_to_char` call with 1–2 array loads; `const fn strip_mark` removes PHF lookups from the spelling inner loop.
- **Hot-Path Cleanup:** Eliminated double `update_cached_output` on the slow path (~2x flatten cost), replaced heap `flatten_slice` with stack `uho_tail_match_composition`, hoisted `get_applicable_rules` and `extract_cvc_trans`, replaced the `[bool; 16]` case shuffle with a `u16` upper-mask bitmask, and removed the dead `uoh_tail_match`.

### Bug Fixes
- **Tone on Literal `oo` (VO_2):** Typing `thooongf` now yields `thoòng` — English bypass is gated on `is_valid` after mark/tone undo so words with literal `oo` still accept tones.
- **Tone Target Resolution:** `find_tone_target` uses absolute vowel indices and resolved mark chains so an undone `ô` no longer steals the tone target.
- **`extract_cvc_appending_indices`:** Fixed absolute-index bug (`results[fc[0]]` → `results[0]`).
- **DFA Hash Collisions:** `add_state` now backward-scans for previously inserted compositions instead of overwriting on collision.

### Testing & Infrastructure
- **Criterion Benchmarks:** New `benches/criterion_bench.rs` with `black_box`-sealed hot paths.
- **Allocation Regression Suite:** `tests/alloc_regression.rs` asserts zero heap allocations across 7 scenarios with a counting allocator.
- **Skey Parity Suite:** 16 tests ported from skey-engine (`tests/skey_parity.rs`), all passing.
- **Dependency Updates:** skey-engine 0.1.4 → 0.1.23, uvie 2.1.1 → 2.7.0.

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
