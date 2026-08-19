# Changelog

All notable changes to this project will be documented in this file.

## [0.3.20] - 2026-08-19

### Performance & Extreme Optimizations
- **L1 Cache-Line 0 Packing (`State` Struct Alignment):** Restructured `State` (160 bytes) via SoA hot/cold field splitting. Packed `bitset`, `trans_keys`, `trans_len`, `comp_hash`, and `comp_offset` into exactly the first 64 bytes (Cache Line 0). Resolves DFA transition lookups in a single L1 cache line access.
- **Fast-Path Case Branching:** Bypassed case-copy loops in DFA cache hits when typing lowercase text (95%+ common keystrokes), reducing hot path to a single direct `memcpy`.
- **Zero-Overhead ASCII Character Pushing:** Inlined `push_char_fast` in `flattener`, writing single-byte ASCII codepoints directly without UTF-8 encoding branch overhead.
- **Vectorized UTF-8 Prefix Diff Counting:** Vectorized UTF-8 leading byte counting (`(b & 0xC0) != 0x80`) in `lcp_chars_and_bytes`, enabling LLVM SIMD popcount.
- **In-Place Tone Relocation:** `refresh_last_tone_target_into` now mutates composition slices directly in-place, eliminating temporary stack buffer copies.
- **Fat LTO Benchmark Configuration:** Configured `[profile.bench]` with `lto = "fat"` and `codegen-units = 1`.

### Correctness & Phonotactics
- **DFA Canonical Lowercase Isolation:** DFA Arena now stores canonical lowercase states exclusively, dynamically mapping case on the stack to prevent $2^N$ state explosion.
- **Auto-Correct Undo Protection:** Protected explicit user undos from being incorrectly restored by auto-correct while allowing standard English word auto-correction.
- **Companion Horn Onset Guard (`is_th_or_h_open`):** Open syllables with `th` or `h` onsets (`thuở`, `huơ`) only horn `o \to ơ` without companion horning `u \to ư`.

### Test Suites
- Added comprehensive cross-validation test suites ported from `skey-engine`, `uvie` (v2.1.1), and `vi` (v0.8.0):
  - `tests/skey_comparison_suite.rs` (226 baseline Vietnamese tests)
  - `tests/skey_repo_tests.rs` (10 test groups covering all 70 vowel sequences, rapid tone reassignment, and zero-delay typing streams)
  - `tests/vi_uvie_comprehensive_suite.rs` (8 test groups covering freestyle typing, full sentences, and phonotactic constraints)

## [0.3.19] - 2026-08-18

### Fixed
- **Non-adjacent repeated tone keys (`looixfsx`):** `is_effective` now compares against the latest transformation of the same effect type on the target character instead of the first match. A new tone key always overrides the previous tone, so `looixfsx` → `lỗi` and `loixfx` → `lõi` (previously the stale first match made the engine fall back to undo + literal key: `lôix`).
- **Adjacent duplicate tone keys preserved as undo:** Typing the same tone key twice in a row (`ss`, `xx`, ...) still removes the tone and types the key literally (`ass` → `as`, `loiss` → `lois`).

### Added
- Keystroke regression tests: `test_telex_non_adjacent_repeated_tone_keys` and `test_telex_adjacent_tone_key_undo` in `tests/keystroke_verification.rs`.
- New `benches/vi_bench.rs` benchmark comparing against `skey-engine`, `uvie`, and `vi` crates.

### Docs
- Added "Tone Key Semantics" section to crate docs and README.
- Documented tone key override/undo behavior on `Engine::process_key`.

## [0.3.18] - 2026-08-15

### Performance
- **SWAR 8-byte Vectorized DFA Lookup:** Replaced linear loop in `get_transition` with SWAR (SIMD Within A Register) on 8-byte chunks (`u64` XOR + `wrapping_sub` bitmask). Resolves transitions in O(1) branchless instructions. Single word benchmarks improved up to 5.6× over uvie (e.g. `vietj -> việt` at 43.5 ns).
- **Structure-of-Arrays (SoA) DFA State:** Split `transitions: [(u8, u32); 24]` (192 bytes) into dense `trans_keys: [u8; 24]` (24 bytes) + `trans_states: [u32; 24]` (96 bytes). Reduces `State` size from 232 → 160 bytes (saves ~300 KB RAM for 4,200 states and fits key array in a single cache line).
- **Struct Memory Layout Optimization:** Reordered fields in `Rule` (28 → 24 bytes) and `Transformation` (32 → 28 bytes). Total `warm_up()` memory dropped from 5.8 MB → 4.7 MB (>1.1 MB saved), `Engine::new()` allocation reduced from 61.2 KB → 52.2 KB.
- **8-byte Word LCP Diffing:** `lcp_chars_and_bytes` now compares 8-byte words via `u64` XOR and `trailing_zeros`, with fast-path ASCII check bypassing UTF-8 `chars().count()`. Editor Delta API speedup of ~37% (617 ns → 389 ns).
- **Branchless ASCII Fast Paths:** Added ASCII fast-paths to `find_vowel_position`, `find_tone_from_char`, `add_tone_to_char`, `add_mark_to_toneless_char`, and `is_vietnamese_rune`, completely bypassing PHF map hashing for standard ASCII keystrokes.
- **Stack Footprint Reduction in Flattener:** Replaced `[Option<usize>; 16]` arrays (896 bytes) in `write_canvas_slice` with compact `u8` sentinel arrays (64 bytes), initialized via a single 128-bit store instruction.
- **Spelling Token Loop Unrolling:** Specialized `lookup_mask_optimized` for length 1 and length 2 queries, eliminating iterator zip overhead during syllable validation.
- **Eliminated Redundant Linear Searches:** Added `find_last_appending_entry` to directly return index and transformation in one pass, eliminating repeated `.position()` scans in `bamboo_util`.

### Added
- **Parallel Batch Processing Module:** Added `parallel` feature flag enabling `bamboo_core::parallel::process_batch` with `rayon` work-stealing for high-throughput batch text and dataset transformation.
- **Release Build Profiles:** Added tuned `[profile.release]` and `[profile.bench]` configurations in `Cargo.toml` with LTO, codegen-units = 1, and panic = "abort".

## [0.3.17] - 2026-08-09


### Performance
- **Compact DFA State (520 → 92 bytes):** Replaced `[u32; 128]` transitions with sorted `[(u8, u32); 24]` + 128-bit bitset. State size reduced 5.7×, improving CPU cache utilization for ~4,200 DFA states.
- **Hash cache:** Added `comp_hash` field to State, replaced `composition_to_state: FxHashMap<Box<[T]>, u32>` with `hash_to_state: FxHashMap<u64, u32>`. Eliminates ~4,200 Box heap allocations.
- **`generate_transformations` zero-alloc:** Replaced `flatten_slice` + string search with direct composition scan (`contains_uho_in_composition`). Eliminates 1 String allocation per slow-path keystroke.
- **Byte-level `lcp_chars_and_bytes`:** Replaced `chars().zip()` with byte-level comparison for delta API. Avoids per-char UTF-8 decoding.
- **`copy_from_slice` in backspace:** Replaced per-element loop with memcpy for tone refresh in `remove_last_char` and `remove_last_output_char`.

### Changed
- `input_method()` now returns `&InputMethod` instead of cloning.
- `to_flags()`, `from_flags()`, `Config::new()` are now `const fn`.
- All clippy warnings resolved (0 warnings).
- Iterator patterns: `.iter().all()`, `.position()`, `.find()`, `.zip().all()`, `.is_some_and()`, `.map_or()` throughout.

### Memory
- `Engine::new()`: 98 KB → 61 KB (1.6× smaller)
- `warm_up()`: 11.4 MB → 5.7 MB (2.0×), allocs 9,316 → 1,164 (8× fewer)
- 10K Vietnamese sentences: 8.1 MB → 5.6 MB (1.4×)

## [0.3.16] - 2026-08-09

### Performance
- **`commit()` zero-heap:** Eliminated temporary `String` allocation in `commit()`. Now writes flattened output directly into `committed_text` via `append_flatten_slice()`. Result: 10K Vietnamese sentences — allocs reduced from 229,306 → 306 (**756× fewer**), heap from 9.9 MB → 8.1 MB. 10K English — allocs from 260,026 → 27.
- **`compile_common()` zero-heap:** Replaced `String::with_capacity(8)` per syllable with stack-allocated `[u8; 8]` buffer. warm_up() allocations reduced from 13,366 → 9,316 (**4,050 fewer**).
- **`get_processed_str()` no-clone fast path:** When `active` is empty and `FULL_TEXT` is set, returns `committed_text.clone()` directly instead of cloning + appending empty string.

### Internal
- Added `flattener::append_flatten_slice()` — appends to existing String without clearing.

## [0.3.15] - 2026-08-09

### Docs
- Removed benchmark tables from README (available in git history).

## [0.3.14] - 2026-08-09

### Performance
- **Uppercase DFA caching:** Uppercase keys (e.g. `TIEENGS`) now share the same DFA cache as lowercase. Previously, all uppercase keys bypassed the DFA fast path entirely, causing 50–106× slowdown. After fix: uppercase is ~1.0× (same as lowercase).
  - `AA → Â`: 1,421 ns → **13.5 ns** (105× faster)
  - `NGUOWIF → NGƯỜI`: 7,807 ns → **74.3 ns** (105× faster)
  - `TIEENGS` (all uppercase): 7,307 ns → **80.5 ns** (91× faster)
- **`warm_up()` memory reduction:** Reuse a single `Engine` instance during DFA pre-compilation instead of creating a new one per syllable. Allocations reduced from **2,544 MB → 11.4 MB** (223× less).
- **Smaller `Engine::new()` allocation:** Reduced DFA pre-allocation from 1,024 → 128 states, committed_text from 256 → 128 bytes, preedit buffers from 64 → 32 bytes. `Engine::new()` now allocates **98 KB** instead of 651 KB (6.6× smaller).

### Added
- **`Engine::dfa_state_count()`** — returns the number of DFA states currently cached.
- **`Engine::dfa_arena_len()`** — returns the number of Transformations in the DFA arena.
- **`Engine::dfa_composition_count()`** — returns the number of entries in the DFA composition-to-state map.
- **`Engine::committed_text_capacity()`** — returns the capacity of the committed text buffer.

### Internal
- `DfaCompiler` now owns an `Engine` instance and resets it per syllable (no repeated allocation).
- DFA fast path uses `lower_key` for lookup; uppercase flag applied post-copy on first transformation.
- DFA JIT cache normalizes compositions to lowercase so uppercase/lowercase share transitions.

## [0.3.13] - 2026-08-05

### Added
- **`Engine::remove_last_output_char()`** — grapheme-level backspace: deletes the entire output character before the caret, keeping mark/tone transformations on earlier characters. Unlike `remove_last_char` (which undoes keystrokes one at a time via snapshot stack), this removes the whole grapheme in a single operation.
- **FFI export:** `bamboo_engine_remove_last_output_char(engine)` for C consumers.

### Internal
- Dropped redundant `unsafe` blocks in FFI module via module-level `#[allow(unused_unsafe)]`.

## [0.3.12] - 2026-07-12

### Performance
- **FxHashMap for DFA state lookup:** Replaced `std::collections::HashMap` with `rustc_hash::FxHashMap` for the DFA composition-to-state map. FxHash is significantly faster than SipHash for the short `Box<[Transformation]>` keys used in JIT path. Benchmark improvements: DFA miss path -13%, mixed typing -15%, random typing -18%, commit via space -22%.
- **Scratch engine reuse in `restore_last_word`:** Added a lazily-initialized `scratch_engine` field to avoid creating a new `Engine::with_config` (~19 µs) on every `restore_last_word` call.

### Internal
- `Dfa` now implements `Clone`.
- Code formatting aligned with `rustfmt`.

## [0.3.11] - 2026-07-09

### Docs
- Clarified `process_key_delta` as 3-way diff API: `(backspace_count, backspaces_bytes, inserted_suffix)`.
- Added contract explanation and examples in rustdoc and README.

## [0.3.10] - 2026-07-09

### Docs
- Rewrote crate-level docs: `process_key` as primary API, `process` as convenience.
- Added API overview table for docs.rs.

## [0.3.9] - 2026-07-09

### Performance
- **English mode optimized:** Skip DFA lookup, direct buffer append. Long identifiers 18.5× faster than uvie.

### Docs
- Added benchmark comparison table (bamboo-core vs uvie) in README.
- New real-world benchmarks: english passthrough, long identifier, backspace spam, commit latency, worst-case syllable, random typing.

## [0.3.8] - 2026-07-09

### Docs
- Rewrote README with usage examples (incremental processing, delta updates, backspace, output customization).

## [0.3.7] - 2026-07-09

### Performance
- **O(1) Backspace via Snapshot Stack (~11,130ns → ~144ns, 77× faster):** Replaced the O(n) replay-based `remove_last_char` with a stack-allocated snapshot approach. Before each keystroke mutates the composition, the engine saves a lightweight snapshot (active buffer + state id). On backspace, the previous state is restored in O(1) via `memcpy`. All snapshots are stack-allocated (`[Snapshot; 16]`), zero heap allocation.

### Internal
- Added `Snapshot` struct (private) for backspace state management.
- `process_key()`: pushes snapshot before DFA fast path (if buffer non-empty) and before slow-path mutations.
- `commit()` / `reset()`: clear snapshot stack.
- `remove_last_char()`: rewritten to use `pop_snapshot()` instead of replaying keystrokes through a temporary engine.

## [0.3.6] - 2026-07-03

### Performance
- **DFA Fast Path 2× Faster (~100ns → ~45ns):** Reorganized `process_key` into three distinct paths (English → DFA fast → slow). The DFA fast path now skips `can_process_key_raw` validation entirely since cached transitions guarantee key validity.
- **Compact Transformation Struct (48 → 28 bytes):** Changed `target` field from `Option<usize>` (16 bytes) to `Option<u8>` (2 bytes). Since `MAX_ACTIVE_TRANS = 16`, `u8` is sufficient. This reduces `[Transformation; 16]` from 768 to 448 bytes (42% smaller), improving CPU cache utilization.
- **Pre-allocated `committed_text`:** Now pre-allocates 256 bytes to avoid reallocation during typical Vietnamese text input.

### Changed
- `Transformation.target` type: `Option<usize>` → `Option<u8>`
- `find_root_target()` signature: `usize` → `u8` for target parameter and return
- `find_tone_target()`, `find_mark_target()`, `find_target()`: return types use `u8` instead of `usize`

### Internal
- Added `benches/engine_bench.rs` with benchmarks for DFA fast path, DFA miss path, output, process_key_delta, mixed typing, backspace, and many words scenarios.

## [0.3.5] - 2026-07-03

### Performance
- Zero-allocation hot path using stack-allocated `TransformationStack`
- Lazy JIT DFA engine with dynamic state caching
- Arena-based DFA state storage

### Fixed
- Robust state recovery in `remove_last_char` and backspace logic
- DFA state and transformation target synchronization

## [0.3.3] - 2026-06-01

### Added
- `Engine::warm_up()` for DFA pre-compilation
- High-performance single-pass O(N) spelling validation

### Performance
- 20× performance improvement through DFA caching (~490µs → ~23µs per cycle)

## [0.3.0] - 2026-05-01

### Added
- `process_key_delta()` for efficient IME text updates
- `OutputOptions` bitflags for output customization
- `Config` struct for engine configuration
- FFI layer for C/C++ integration
- WASM bindings via `wasm-bindgen`

### Changed
- Major refactor of engine architecture
- New transformation-based input processing model

## [0.2.1] - 2026-04-01

### Fixed
- Various input method parsing issues
- Documentation improvements

## [0.1.2] - 2026-03-01

### Added
- FFI layer for external integration
- Initial documentation

## [0.1.1] - 2026-02-01

### Added
- Initial release
- Telex, VNI, VIQR input methods
- Unicode support
- Basic engine functionality
