# Bamboo Core (Rust)

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Vietnamese input method engine (IME) core written in Rust, ported from [bamboo-core](https://github.com/BambooEngine/bamboo-core) (Go).

## Features

- **Telex, VNI, VIQR** input methods with custom input method support
- **Hybrid engine**: Rule-based transformations + Lazy JIT DFA caching
- **Zero heap allocation** in core processing path (stack-allocated buffers)
- **O(1) backspace** via snapshot stack
- **O(N) single-pass** spelling validation
- **FFI** for C/C++ integration and **WASM** bindings

## Installation

```toml
[dependencies]
bamboo-core = "0.3.14"
```

## Quick Start

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

let word = engine.process("tieengs", Mode::Vietnamese);
assert_eq!(word, "tiếng");

engine.reset();
let word2 = engine.process("vieetj", Mode::Vietnamese);
assert_eq!(word2, "việt");
```

## Incremental Processing

Process keystrokes one at a time for IME integration:

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

### Delta Updates

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
previous = [common_prefix] + [backspace_count chars to delete]
new      = [common_prefix] + [inserted_suffix]
```
Frontend does not need to compute LCP — the engine does it.

## Backspace

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("chuyeenr", Mode::Vietnamese);
assert_eq!(engine.output(), "chuyển");

engine.remove_last_char(true);
assert_eq!(engine.output(), "chuyên");
```

Two backspace modes are available:

- `remove_last_char(true)` — undo the **last keystroke** (O(1) via snapshot stack).
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

## Benchmarks

Benchmarked against [uvie](https://github.com/thuupx/uvie-rs) v2.1.1 and [skey-engine](https://github.com/collyn/skey-engine) v0.1.4 on the same machine. Lower is better.

### Core Operations (vs uvie)

| Benchmark | Bamboo | Uvie | Speedup |
|---|---:|---:|---:|
| feed single word (tieengs) | 73 ns | 782 ns | **10.7x** |
| feed + delta (per key) | 706 ns | 787 ns | **1.1x** |
| mixed typing (29 chars) | 758 ns | 3,293 ns | **4.3x** |
| many words (13 words) | 5,871 ns | 10,530 ns | **1.8x** |

### Backspace (vs uvie)

| Benchmark | Bamboo | Uvie | Speedup |
|---|---:|---:|---:|
| backspace (1x) | 144 ns | 953 ns | **6.6x** |
| backspace x3 | 275 ns | 1,204 ns | **4.4x** |
| backspace spam (7x) | 425 ns | 1,459 ns | **3.4x** |

### Uppercase Performance (vs skey-engine)

Since v0.3.14, uppercase keys share the same DFA cache as lowercase.

| Benchmark | Bamboo | skey | Ratio |
|---|---:|---:|---:|
| `AA → Â` (2 chars) | 13.5 ns | 29.9 ns | **0.45x** (faster) |
| `VIETJ → VIỆT` (5 chars) | 44.5 ns | 52.3 ns | **0.85x** (faster) |
| `TIEENGS` (7 chars, uppercase) | 80.5 ns | 61.2 ns | 1.31x |
| `NGUOWIF → NGƯỜI` (7 chars) | 74.3 ns | 65.9 ns | 1.13x |

### Vietnamese Words (vs skey-engine)

| Benchmark | Bamboo/key | skey | Ratio |
|---|---:|---:|---:|
| `tieengs → tiếng` | 76.5 ns | 57.7 ns | 1.32x |
| `vietj → việt` | 39.1 ns | 53.0 ns | **0.74x** (faster) |
| `khongf → không` | 54.8 ns | 53.0 ns | 1.03x |
| 202 syllables (avg) | 24.2 ns | 46.4 ns | **0.52x** (faster) |

### Scalability (vs skey-engine)

| Input length | Bamboo/key | skey | Ratio |
|---|---:|---:|---:|
| 1 word (7 chars) | 68.6 ns | 58.7 ns | 1.17x |
| 5 words (36 chars) | 1,077 ns | 242 ns | 4.45x |
| 10 words (77 chars) | 2,579 ns | 505 ns | 5.11x |
| 15 words (109 chars) | 3,861 ns | 734 ns | 5.26x |
| Paragraph (285 chars) | 9,236 ns | 2,002 ns | 4.61x |

### Real-world Scenarios (vs uvie)

| Benchmark | Bamboo | Uvie | Speedup |
|---|---:|---:|---:|
| english passthrough (code) | 351 ns | 1,289 ns | **3.7x** |
| long identifier (39 chars) | 557 ns | 9,528 ns | **17.1x** |
| commit via space | 131 ns | 712 ns | **5.4x** |
| worst-case syllable (nghieengs) | 98 ns | 904 ns | **9.2x** |
| random typing (real workload) | 3,202 ns | 7,129 ns | **2.2x** |

### Memory Profile

| Metric | Value |
|---|---:|
| `Engine::new()` allocation | 98 KB |
| `Engine` struct size | 9.6 KB (stack) |
| `warm_up()` allocation | 11.4 MB |
| DFA states after warmup | 4,077 |
| RAM for 10,000 words | ~70 KB |

> **Note**: Bamboo's cold start is slower (~27 µs) due to Engine + DFA allocation. After warmup, the DFA fast path dominates. Since v0.3.14, uppercase keys also benefit from DFA caching.

Run benchmarks: `cargo bench`

## Credits

- **Rust Port & Optimization:** Dao Trong Nguyen ([@nguyen10t2](https://github.com/nguyen10t2))
- **Original Author (Go):** Lam ([@lamtq](https://github.com/t1ld3x))
- **Technical Consultant:** Mai Tan Phat ([@phatMT97](https://github.com/phatMT97)) - Author of **VKey**

## License

MIT. See [LICENSE](LICENSE).
