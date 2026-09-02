# Bamboo Core (Rust)

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Vietnamese input method engine (IME) core written in Rust, ported from [bamboo-core](https://github.com/BambooEngine/bamboo-core) (Go).

## Features

- **Telex, VNI, VIQR** input methods with custom input method support
- **Hybrid engine**: Rule-based transformations + Lazy JIT DFA caching
- **SWAR / Vectorized matching**: Branchless 8-byte word lookups for transition routing
- **Parallel batch processing**: Rayon-powered work-stealing for bulk text processing (`parallel` feature)
- **Zero heap allocation** in core processing path (stack-allocated buffers)
- **O(1) backspace** via snapshot stack
- **O(N) single-pass** spelling validation
- **FFI** for C/C++ integration and **WASM** bindings

## Installation

```toml
[dependencies]
bamboo-core = "0.3.23"
```

## Quick Start — IME Integration

Feed keystrokes one at a time with `process_key`:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

engine.process_key('t', Mode::Vietnamese);
engine.process_key('i', Mode::Vietnamese);
engine.process_key('e', Mode::Vietnamese);
engine.process_key('e', Mode::Vietnamese);
engine.process_key('n', Mode::Vietnamese);
engine.process_key('g', Mode::Vietnamese);
engine.process_key('s', Mode::Vietnamese);
assert_eq!(engine.output(), "tiếng");
```

### Batch & Testing Convenience

> ⚠️ **Note:** `Engine::process` is a convenience wrapper intended for **testing and batch validation only** (not for real-time production IME integration). For production applications, use `process_key` or `process_key_delta`.

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

// Testing convenience wrapper
let word = engine.process("tieengs", Mode::Vietnamese);
assert_eq!(word, "tiếng");

engine.reset();
let word2 = engine.process("vieetj", Mode::Vietnamese);
assert_eq!(word2, "việt");
```

## Tone Key Semantics

In Telex, a new tone key always replaces the previous tone on the same vowel —
the last tone key wins, even when tone keys are interleaved with letters or
other tone keys:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());
assert_eq!(engine.process("looixfsx", Mode::Vietnamese), "lỗi");
```

Typing the same tone key twice in a row is an intentional undo: the tone is
removed and the key is typed as a literal letter.

```rust
let mut engine = Engine::new(InputMethod::telex());
assert_eq!(engine.process("ass", Mode::Vietnamese), "as");
```

## Delta Updates for Text Editors

For efficient text editor integration, use `process_key_delta` to get a **3-way diff**:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

let (bs, _, ins) = engine.process_key_delta('a', Mode::Vietnamese);
assert_eq!(bs, 0);
assert_eq!(ins, "a");

// previous = "a", new = "á"
let (bs, _, ins) = engine.process_key_delta('s', Mode::Vietnamese);
assert_eq!(bs, 1);     // delete 1 char ("a")
assert_eq!(ins, "á");  // insert "á"
// result: "" + "á" = "á"
```

Contract:
```text
previous = [common_prefix] + [backspace_count chars to delete]\nnew      = [common_prefix] + [inserted_suffix]
```
Frontend does not need to compute LCP — the engine does it.

## Backspace

```rust
use bamboo_core::{Engine, Mode, InputMethod, RestoreMark};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("chuyeenr", Mode::Vietnamese);
assert_eq!(engine.output(), "chuyển");

engine.remove_last_char(RestoreMark::Yes); // or pass true
assert_eq!(engine.output(), "chuyên");
```

Two backspace modes are available:

- `remove_last_char(RestoreMark::Yes)` (or `true`) — undo the **last keystroke** (O(1) via snapshot stack).
  `tiếng` + DEL -> `tiêng`.
- `remove_last_output_char()` — delete the **entire character before the caret**,
  keeping mark/tone transformations on earlier characters. `tiếng` + DEL -> `tiến`:

```rust
let mut engine = Engine::new(InputMethod::telex());
engine.process_str("tieesng", Mode::Vietnamese);
assert_eq!(engine.output(), "tiếng");

engine.remove_last_output_char();
assert_eq!(engine.output(), "tiến");
```

## Output Customization

```rust
use bamboo_core::{Engine, Mode, InputMethod, OutputOptions};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("Trangws", Mode::Vietnamese);

// Toneless
assert_eq!(engine.get_processed_str(OutputOptions::TONE_LESS), "Trăng");

// Full text (committed + active)
assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "Trăng");
```

## Performance & Benchmarks

Bamboo Core is architected for zero heap allocations in the interactive typing loop, sub-microsecond keystroke latency, and high CPU cache efficiency via L1 cache line packing and SWAR vector matching.

| Benchmark Scenario | Previous Baseline (`v0.3.19`) | Optimized (`v0.3.21`) | Speedup |
|---|---|---|---|
| Compound Word (`nghiengs`) | 54.34 ns/op | **43.79 ns/op** | **+19.4%** |
| Random Keystroke Sequence | 1.100 µs/op | **948.1 ns/op** | **+13.8%** |
| Worst-case Deep Syllable | 70.36 ns/op | **62.77 ns/op** | **+10.8%** |
| Mixed Typing (Viet + English) | 6.545 µs/op | **5.748 µs/op** | **+12.2%** |
| Rapid Backspace Burst | 277.6 ns/op | **256.4 ns/op** | **+7.6%** |
| English Passthrough | 49.33 ns/op | **47.66 ns/op** | **+3.4%** |
| Feed Benchmark | 88.08 ns/op | **87.21 ns/op** | **+1.0%** |

*Benchmarks measured on x86_64 Linux, Rust 1.80+ release profile with Fat LTO.*

## Architecture

- **`Engine`**: Core state machine managing character buffer, keystroke history snapshots, and output formatting.
- **`Dfa`**: JIT cache with Arena Allocation, 128-bit bitset fast rejection, and SWAR 8-byte chunk scanning for $O(1)$ state transitions.
- **`InputMethod`**: Transformation rules for Telex, VNI, VIQR, and Microsoft layout with multi-character expansions.
- **`Config`**: Runtime toggles for free tone placement, modern/traditional tone style, and auto-correct.
- **`encoder`**: Zero-allocation encoding conversion supporting 16 Vietnamese legacy charsets (TCVN3, VNI-Windows, VIQR, VISCII, etc.).
- **`ffi`**: C-ABI bindings for integration with C, C++, Python, Fcitx5, IBus, and native GUI toolkits.

## License

MIT License. See [LICENSE](LICENSE) for details.
